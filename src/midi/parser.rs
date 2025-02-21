use super::timing::ticks_to_ms;
use super::types::{
    Channel, MidiNote, NoteEvent, ProcessedSong, SoundFontMap, TempoChange, TempoMap, Timestamp,
    Velocity,
};
use midly::{Smf, TrackEventKind};
use std::collections::HashMap;
use std::error::Error;

const DRUM_CHANNEL: u8 = 9; // MIDI channel 10 (0-based)

/// Parses a MIDI file and extracts note events and channel information.
///
/// # Arguments
/// * `midi_data` - Raw MIDI file data
/// * `info_only` - If true, only parse channel information and return empty note events
///
/// # Returns
/// * `ProcessedSong` - Parsed MIDI data including notes, channels, and a dummy soundfont
///
/// # Errors
/// * If the MIDI file is invalid
/// * If the timing format is unsupported
pub fn parse_midi(midi_data: &[u8], info_only: bool) -> Result<ProcessedSong, Box<dyn Error>> {
    let smf = Smf::parse(midi_data)?;

    let ticks_per_quarter = match smf.header.timing {
        midly::Timing::Metrical(ticks) => ticks.as_int() as u32,
        _ => return Err("Unsupported timing format".into()),
    };

    // Initialize tempo map and collect all events
    let mut tempo_map = TempoMap::new(ticks_per_quarter);
    let mut all_events = Vec::new();
    let mut tempo_changes = Vec::new();
    let mut channels = HashMap::new();
    let mut channel_instruments = HashMap::new();

    // First pass: collect all events and tempo changes with absolute timestamps
    for track in smf.tracks.iter() {
        let mut track_time: u64 = 0;
        for event in track {
            track_time += u64::from(event.delta.as_int());
            match event.kind {
                TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo_val)) => {
                    tempo_changes.push(TempoChange {
                        tick: track_time,
                        tempo: tempo_val.as_int(),
                    });
                }
                TrackEventKind::Midi { channel, message } => {
                    let ch = channel.as_int();
                    // Record any channel that has MIDI messages
                    if !channels.contains_key(&ch) {
                        channels.insert(
                            ch,
                            Channel {
                                id: ch,
                                instrument: *channel_instruments.get(&ch).unwrap_or(&0), // Default to piano
                                is_drum: ch == DRUM_CHANNEL,
                            },
                        );
                    }
                    // Update instrument if we see a program change
                    if let midly::MidiMessage::ProgramChange { program } = message {
                        channel_instruments.insert(ch, program.as_int());
                        if let Some(channel) = channels.get_mut(&ch) {
                            channel.instrument = program.as_int();
                        }
                    }
                    all_events.push((track_time, ch, message));
                }
                _ => {}
            }
        }
    }

    // Convert channels HashMap to Vec, sorted by channel ID
    let mut channel_vec: Vec<Channel> = channels.into_values().collect();
    channel_vec.sort_by_key(|c| c.id);

    if info_only {
        return Ok(ProcessedSong {
            note_changes: Vec::new(),
            channels: channel_vec,
            soundfonts: SoundFontMap::new(vec![vec![1.0]]), // Dummy soundfont
        });
    }

    // Sort and merge tempo changes, ensuring they are in chronological order
    tempo_changes.sort_by_key(|change| change.tick);

    // Merge tempo changes that occur at the same tick, keeping the last one
    let mut last_tick = None;
    for change in tempo_changes {
        match last_tick {
            Some(tick) if tick == change.tick => {
                // Replace the last tempo change at this tick
                if let Some(last_change) = tempo_map.changes.last_mut() {
                    last_change.tempo = change.tempo;
                }
            }
            _ => {
                tempo_map.changes.push(change.clone());
                last_tick = Some(change.tick);
            }
        }
    }

    // Sort note events by timestamp
    all_events.sort_by_key(|(time, _, _)| *time);

    // Process the merged events
    let mut active_notes: HashMap<(MidiNote, u8), (Velocity, Timestamp)> = HashMap::new(); // (Note, Channel) -> (Velocity, Start Time)
    let mut note_changes: HashMap<Timestamp, Vec<(MidiNote, Velocity, usize, Timestamp)>> =
        HashMap::new();

    // Find the last MIDI event time for proper song duration
    let last_event_time = all_events
        .last()
        .map(|(time, _, _)| ticks_to_ms(*time, &tempo_map))
        .unwrap_or(0);

    for (track_time, channel, message) in all_events {
        let current_time = ticks_to_ms(track_time, &tempo_map);

        match message {
            midly::MidiMessage::NoteOn { key, vel } => {
                let note_key = (key.as_int(), channel);
                if vel.as_int() > 0 {
                    // Note on - record start time
                    active_notes.insert(note_key, (vel.as_int(), current_time));
                } else {
                    // Note off - if note was active, add it to changes with its duration
                    if let Some((velocity, start_time)) = active_notes.remove(&note_key) {
                        let changes = note_changes.entry(start_time).or_default();
                        changes.push((key.as_int(), velocity, channel as usize, current_time));
                    }
                }
            }
            midly::MidiMessage::NoteOff { key, .. } => {
                // Explicit note off - if note was active, add it to changes with its duration
                let note_key = (key.as_int(), channel);
                if let Some((velocity, start_time)) = active_notes.remove(&note_key) {
                    let changes = note_changes.entry(start_time).or_default();
                    changes.push((key.as_int(), velocity, channel as usize, current_time));
                }
            }
            _ => {}
        }
    }

    // Handle any still-active notes by ending them at the last event time
    for ((note, channel), (velocity, start_time)) in active_notes {
        let changes = note_changes.entry(start_time).or_default();
        changes.push((note, velocity, channel as usize, last_event_time));
    }

    let mut events: Vec<NoteEvent> = note_changes
        .into_iter()
        .map(|(timestamp, notes)| NoteEvent { timestamp, notes })
        .collect();
    events.sort_by_key(|event| event.timestamp);

    Ok(ProcessedSong {
        note_changes: events,
        channels: channel_vec,
        soundfonts: SoundFontMap::new(vec![vec![1.0]]), // Will be replaced by parse_midi_with_soundfonts
    })
}

/// Parses a MIDI file with soundfont information.
///
/// This function first parses the MIDI file normally, then updates the note events
/// to use the provided soundfonts based on the channel mapping.
///
/// # Arguments
/// * `midi_data` - Raw MIDI file data
/// * `soundfonts` - Vector of soundfonts to use
/// * `channel_to_index` - Mapping from channel numbers to soundfont indices
///
/// # Returns
/// * `ProcessedSong` - Parsed MIDI data with soundfont information
///
/// # Errors
/// * If the MIDI file is invalid
/// * If the timing format is unsupported
pub fn parse_midi_with_soundfonts(
    midi_data: &[u8],
    soundfonts: Vec<Vec<f32>>,
    channel_to_index: Vec<Option<usize>>,
) -> Result<ProcessedSong, Box<dyn Error>> {
    let mut song = parse_midi(midi_data, false)?;

    // Update the soundfont indices in note events using the channel mapping
    // and filter out notes for channels without soundfonts
    for event in &mut song.note_changes {
        event.notes.retain_mut(|(_, _, soundfont_idx, _)| {
            if let Some(new_idx) = channel_to_index[*soundfont_idx] {
                *soundfont_idx = new_idx;
                true
            } else {
                false
            }
        });
    }

    // Remove any events that have no notes after filtering
    song.note_changes.retain(|event| !event.notes.is_empty());

    song.soundfonts = SoundFontMap::new(soundfonts);
    Ok(song)
}
