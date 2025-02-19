use std::error::Error;
use std::collections::HashMap;
use midly::{Smf, TrackEventKind};
use super::types::{Timestamp, MidiNote, Velocity, NoteEvent, ProcessedSong, TempoMap, TempoChange};
use super::timing::ticks_to_ms;

pub fn parse_midi(midi_data: &[u8]) -> Result<ProcessedSong, Box<dyn Error>> {
    let smf = Smf::parse(midi_data)?;

    let ticks_per_quarter = match smf.header.timing {
        midly::Timing::Metrical(ticks) => ticks.as_int() as u32,
        _ => return Err("Unsupported timing format".into()),
    };

    // Initialize tempo map and collect all events
    let mut tempo_map = TempoMap::new(ticks_per_quarter);
    let mut all_events = Vec::new();
    let mut tempo_changes = Vec::new();

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
                TrackEventKind::Midi { message, .. } => {
                    all_events.push((track_time, message));
                }
                _ => {}
            }
        }
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
    all_events.sort_by_key(|(time, _)| *time);

    // Process the merged events
    let mut active_notes: Vec<(MidiNote, Velocity)> = Vec::new();
    let mut note_changes: HashMap<Timestamp, Vec<(MidiNote, Velocity)>> = HashMap::new();

    for (track_time, message) in all_events {
        match message {
            midly::MidiMessage::NoteOn { key, vel } => {
                if vel.as_int() > 0 {
                    // Remove any existing instance of this note
                    active_notes.retain(|(n, _)| *n != key.as_int());
                    // Add the new note with velocity
                    active_notes.push((key.as_int(), vel.as_int()));
                    active_notes.sort_unstable_by_key(|(n, _)| *n);
                } else {
                    // Treat as note off
                    active_notes.retain(|(n, _)| *n != key.as_int());
                }
                let ms = ticks_to_ms(track_time, &tempo_map);
                note_changes.insert(ms, active_notes.clone());
            }
            midly::MidiMessage::NoteOff { key, .. } => {
                active_notes.retain(|(n, _)| *n != key.as_int());
                let ms = ticks_to_ms(track_time, &tempo_map);
                note_changes.insert(ms, active_notes.clone());
            }
            _ => {}
        }
    }

    let mut events: Vec<NoteEvent> = note_changes
        .into_iter()
        .map(|(timestamp, notes)| NoteEvent { timestamp, notes })
        .collect();
    events.sort_by_key(|event| event.timestamp);

    Ok(ProcessedSong { note_changes: events })
}