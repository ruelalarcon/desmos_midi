use super::timing::ticks_to_ms;
use super::types::{
    Channel, MidiError, MidiNote, NoteEvent, ProcessedSong, SoundFontMap, TempoChange, TempoMap,
    Timestamp, Velocity,
};
use midly::{Smf, TrackEventKind};
use std::collections::HashMap;

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
pub fn parse_midi(midi_data: &[u8], info_only: bool) -> Result<ProcessedSong, MidiError> {
    // Parse the MIDI file
    let smf = Smf::parse(midi_data)?;

    // Get the ticks per quarter note from the MIDI header
    let ticks_per_quarter = extract_ticks_per_quarter(&smf)?;

    // Extract tempo changes and channel information
    let (tempo_map, channels, all_events) = extract_midi_metadata(&smf, ticks_per_quarter)?;

    // If we only need channel info, return early with empty notes
    if info_only {
        return Ok(ProcessedSong {
            note_changes: Vec::new(),
            channels,
            soundfonts: SoundFontMap::new(vec![vec![1.0]]), // Dummy soundfont
        });
    }

    // Process note events
    let note_events = process_note_events(all_events, &tempo_map);

    Ok(ProcessedSong {
        note_changes: note_events,
        channels,
        soundfonts: SoundFontMap::new(vec![vec![1.0]]), // Will be replaced by parse_midi_with_soundfonts
    })
}

/// Extracts the ticks per quarter note from the MIDI file header
///
/// # Arguments
/// * `smf` - Parsed MIDI file
///
/// # Returns
/// * `u32` - Ticks per quarter note
///
/// # Errors
/// * If the timing format is unsupported
fn extract_ticks_per_quarter(smf: &Smf) -> Result<u32, MidiError> {
    match smf.header.timing {
        midly::Timing::Metrical(ticks) => Ok(ticks.as_int() as u32),
        _ => Err(MidiError::UnsupportedTimingFormat),
    }
}

/// Extracts tempo changes, channel information, and note events from a MIDI file
///
/// # Arguments
/// * `smf` - Parsed MIDI file
/// * `ticks_per_quarter` - Ticks per quarter note
///
/// # Returns
/// * `(TempoMap, Vec<Channel>, Vec<(u64, u8, midly::MidiMessage)>)` -
///   Tempo map, channels, and note events
fn extract_midi_metadata(
    smf: &Smf,
    ticks_per_quarter: u32,
) -> Result<(TempoMap, Vec<Channel>, Vec<(u64, u8, midly::MidiMessage)>), MidiError> {
    // Initialize data structures
    let mut tempo_map = TempoMap::new(ticks_per_quarter);
    let mut all_events = Vec::new();
    let mut tempo_changes = Vec::new();
    let mut channels = HashMap::new();
    let mut channel_instruments = HashMap::new();

    // First pass: collect all events and tempo changes with absolute timestamps
    collect_events_from_tracks(
        smf,
        &mut tempo_changes,
        &mut all_events,
        &mut channels,
        &mut channel_instruments,
    );

    // Sort and process tempo changes
    process_tempo_changes(&mut tempo_map, &mut tempo_changes);

    // Convert channels HashMap to Vec, sorted by channel ID
    let mut channel_vec: Vec<Channel> = channels.into_values().collect();
    channel_vec.sort_by_key(|c| c.id);

    Ok((tempo_map, channel_vec, all_events))
}

/// Collects events from all tracks in the MIDI file
///
/// # Arguments
/// * `smf` - Parsed MIDI file
/// * `tempo_changes` - Collection to store tempo changes
/// * `all_events` - Collection to store MIDI events
/// * `channels` - Collection to store channel information
/// * `channel_instruments` - Collection to track instruments for each channel
fn collect_events_from_tracks(
    smf: &Smf,
    tempo_changes: &mut Vec<TempoChange>,
    all_events: &mut Vec<(u64, u8, midly::MidiMessage)>,
    channels: &mut HashMap<u8, Channel>,
    channel_instruments: &mut HashMap<u8, u8>,
) {
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
}

/// Processes and merges tempo changes, ensuring they are in chronological order
///
/// # Arguments
/// * `tempo_map` - Tempo map to update
/// * `tempo_changes` - Collection of tempo changes to process
fn process_tempo_changes(tempo_map: &mut TempoMap, tempo_changes: &mut Vec<TempoChange>) {
    // Sort tempo changes by time
    tempo_changes.sort_by_key(|change| change.tick);

    // Merge tempo changes that occur at the same tick, keeping the last one
    let mut last_tick = None;
    for change in tempo_changes.clone() {
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
}

/// Processes note events and converts them to NoteEvent structures.
///
/// This function:
/// 1. Tracks active notes and their start times
/// 2. Converts MIDI ticks to milliseconds
/// 3. Groups notes that start at the same time
/// 4. Sorts events chronologically
///
/// # Arguments
/// * `all_events` - Collection of MIDI events
/// * `tempo_map` - Tempo map for timing conversion
///
/// # Returns
/// * `Vec<NoteEvent>` - Processed note events
fn process_note_events(
    all_events: Vec<(u64, u8, midly::MidiMessage)>,
    tempo_map: &TempoMap,
) -> Vec<NoteEvent> {
    // Active notes being tracked: (Note, Channel) -> (Velocity, Start Time)
    let mut active_notes: HashMap<(MidiNote, u8), (Velocity, Timestamp)> = HashMap::new();

    // Note changes grouped by start time: Start Time -> Vec<(Note, Velocity, Channel, End Time)>
    let mut note_changes: HashMap<Timestamp, Vec<(MidiNote, Velocity, usize, Timestamp)>> =
        HashMap::new();

    // Sort events by time
    let mut sorted_events = all_events;
    sorted_events.sort_by_key(|(time, _, _)| *time);

    // Find the last MIDI event time for proper song duration
    let last_event_time = sorted_events
        .last()
        .map(|(time, _, _)| ticks_to_ms(*time, tempo_map))
        .unwrap_or(0);

    // Process each MIDI event
    for (track_time, channel, message) in sorted_events {
        let current_time = ticks_to_ms(track_time, tempo_map);

        match message {
            midly::MidiMessage::NoteOn { key, vel } => {
                handle_note_on(
                    key.as_int(),
                    vel.as_int(),
                    channel,
                    current_time,
                    &mut active_notes,
                    &mut note_changes,
                );
            }
            midly::MidiMessage::NoteOff { key, .. } => {
                handle_note_off(
                    key.as_int(),
                    channel,
                    current_time,
                    &mut active_notes,
                    &mut note_changes,
                );
            }
            _ => {}
        }
    }

    // Handle any still-active notes by ending them at the last event time
    for ((note, channel), (velocity, start_time)) in active_notes {
        let changes = note_changes.entry(start_time).or_default();
        changes.push((note, velocity, channel as usize, last_event_time));
    }

    // Convert the note_changes map to a sorted vector of NoteEvent objects
    let mut events: Vec<NoteEvent> = note_changes
        .into_iter()
        .map(|(timestamp, notes)| NoteEvent { timestamp, notes })
        .collect();
    events.sort_by_key(|event| event.timestamp);

    events
}

/// Handles a note-on event
///
/// # Arguments
/// * `note` - MIDI note number
/// * `velocity` - Note velocity
/// * `channel` - MIDI channel
/// * `current_time` - Current time in milliseconds
/// * `active_notes` - Collection of active notes being tracked
/// * `note_changes` - Collection of note changes grouped by start time
fn handle_note_on(
    note: u8,
    velocity: u8,
    channel: u8,
    current_time: u64,
    active_notes: &mut HashMap<(MidiNote, u8), (Velocity, Timestamp)>,
    note_changes: &mut HashMap<Timestamp, Vec<(MidiNote, Velocity, usize, Timestamp)>>,
) {
    let note_key = (note, channel);

    if velocity > 0 {
        // Note on - record start time
        active_notes.insert(note_key, (velocity, current_time));
    } else {
        // Note on with velocity 0 is equivalent to note off
        handle_note_off(note, channel, current_time, active_notes, note_changes);
    }
}

/// Handles a note-off event
///
/// # Arguments
/// * `note` - MIDI note number
/// * `channel` - MIDI channel
/// * `current_time` - Current time in milliseconds
/// * `active_notes` - Collection of active notes being tracked
/// * `note_changes` - Collection of note changes grouped by start time
fn handle_note_off(
    note: u8,
    channel: u8,
    current_time: u64,
    active_notes: &mut HashMap<(MidiNote, u8), (Velocity, Timestamp)>,
    note_changes: &mut HashMap<Timestamp, Vec<(MidiNote, Velocity, usize, Timestamp)>>,
) {
    let note_key = (note, channel);

    // If note was active, add it to changes with its duration
    if let Some((velocity, start_time)) = active_notes.remove(&note_key) {
        let changes = note_changes.entry(start_time).or_default();
        changes.push((note, velocity, channel as usize, current_time));
    }
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
) -> Result<ProcessedSong, MidiError> {
    let mut song = parse_midi(midi_data, false)?;

    // Update song with soundfont information
    update_song_with_soundfonts(&mut song, soundfonts, channel_to_index);

    Ok(song)
}

/// Updates a song with soundfont information.
///
/// This function:
/// 1. Updates soundfont indices in note events
/// 2. Removes notes for channels without soundfonts
/// 3. Removes empty note events
/// 4. Updates the song's soundfont map
///
/// # Arguments
/// * `song` - Song to update
/// * `soundfonts` - Vector of soundfonts to use
/// * `channel_to_index` - Mapping from channel numbers to soundfont indices
fn update_song_with_soundfonts(
    song: &mut ProcessedSong,
    soundfonts: Vec<Vec<f32>>,
    channel_to_index: Vec<Option<usize>>,
) {
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
}
