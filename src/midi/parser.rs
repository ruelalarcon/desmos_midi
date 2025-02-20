use std::error::Error;
use std::collections::HashMap;
use midly::{Smf, TrackEventKind};
use super::types::{Timestamp, MidiNote, Velocity, NoteEvent, ProcessedSong, TempoMap, TempoChange, Channel, SoundFontMap};
use super::timing::ticks_to_ms;

const DRUM_CHANNEL: u8 = 9; // MIDI channel 10 (0-based)

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
                    if let midly::MidiMessage::ProgramChange { program } = message {
                        channels.insert(channel.as_int(), Channel {
                            id: channel.as_int(),
                            instrument: program.as_int(),
                            is_drum: channel.as_int() == DRUM_CHANNEL,
                        });
                    }
                    all_events.push((track_time, channel.as_int(), message));
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
    let mut active_notes: Vec<(MidiNote, Velocity, u8)> = Vec::new(); // Note, Velocity, Channel
    let mut note_changes: HashMap<Timestamp, Vec<(MidiNote, Velocity, usize)>> = HashMap::new();

    for (track_time, channel, message) in all_events {
        match message {
            midly::MidiMessage::NoteOn { key, vel } => {
                if vel.as_int() > 0 {
                    // Remove any existing instance of this note on this channel
                    active_notes.retain(|(n, _, ch)| *n != key.as_int() || *ch != channel);
                    // Add the new note with velocity
                    active_notes.push((key.as_int(), vel.as_int(), channel));
                    active_notes.sort_unstable_by_key(|(n, _, _)| *n);
                } else {
                    // Treat as note off
                    active_notes.retain(|(n, _, ch)| *n != key.as_int() || *ch != channel);
                }
                let ms = ticks_to_ms(track_time, &tempo_map);
                // Store channel ID directly, will be mapped to soundfont index later
                let notes: Vec<_> = active_notes.iter()
                    .map(|&(note, vel, ch)| (note, vel, ch as usize))
                    .collect();
                note_changes.insert(ms, notes);
            }
            midly::MidiMessage::NoteOff { key, .. } => {
                active_notes.retain(|(n, _, ch)| *n != key.as_int() || *ch != channel);
                let ms = ticks_to_ms(track_time, &tempo_map);
                let notes: Vec<_> = active_notes.iter()
                    .map(|&(note, vel, ch)| (note, vel, ch as usize))
                    .collect();
                note_changes.insert(ms, notes);
            }
            _ => {}
        }
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

pub fn parse_midi_with_soundfonts(
    midi_data: &[u8],
    soundfonts: Vec<Vec<f32>>,
    channel_to_index: Vec<usize>
) -> Result<ProcessedSong, Box<dyn Error>> {
    let mut song = parse_midi(midi_data, false)?;

    // Update the soundfont indices in note events using the channel mapping
    for event in &mut song.note_changes {
        for (_, _, soundfont_idx) in &mut event.notes {
            *soundfont_idx = channel_to_index[*soundfont_idx];
        }
    }

    song.soundfonts = SoundFontMap::new(soundfonts);
    Ok(song)
}