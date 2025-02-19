use std::error::Error;
use std::collections::HashMap;
use midly::{Smf, TrackEventKind};
use super::types::{Timestamp, MidiNote, NoteEvent, ProcessedSong, TempoMap, TempoChange};
use super::timing::ticks_to_ms;

pub fn parse_midi(midi_data: &[u8]) -> Result<ProcessedSong, Box<dyn Error>> {
    let smf = Smf::parse(midi_data)?;

    let ticks_per_quarter = match smf.header.timing {
        midly::Timing::Metrical(ticks) => ticks.as_int() as u32,
        _ => return Err("Unsupported timing format".into()),
    };

    // Initialize tempo map
    let mut tempo_map = TempoMap::new(ticks_per_quarter);
    let mut tempo_changes = Vec::new();

    // First pass: collect all tempo changes
    for track in smf.tracks.iter() {
        let mut track_time: u64 = 0;
        for event in track {
            track_time += u64::from(event.delta.as_int());
            if let TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo_val)) = event.kind {
                tempo_changes.push(TempoChange {
                    tick: track_time,
                    tempo: tempo_val.as_int(),
                });
            }
        }
    }

    // Sort and merge tempo changes
    tempo_changes.sort_by_key(|change| change.tick);
    for change in tempo_changes {
        if !tempo_map.changes.iter().any(|c| c.tick == change.tick) {
            tempo_map.changes.push(change);
        }
    }

    // Second pass: collect all note events
    let mut all_events = Vec::new();
    for track in smf.tracks.iter() {
        let mut track_time: u64 = 0;
        for event in track {
            track_time += u64::from(event.delta.as_int());
            if let TrackEventKind::Midi { message, .. } = event.kind {
                all_events.push((track_time, message));
            }
        }
    }

    // Sort events by timestamp
    all_events.sort_by_key(|(time, _)| *time);

    // Process the merged events
    let mut active_notes = Vec::new();
    let mut note_changes: HashMap<Timestamp, Vec<MidiNote>> = HashMap::new();

    for (track_time, message) in all_events {
        match message {
            midly::MidiMessage::NoteOn { key, vel } => {
                if vel.as_int() > 0 {
                    active_notes.push(key.as_int());
                    active_notes.sort_unstable();
                } else {
                    active_notes.retain(|&x| x != key.as_int());
                }
                let ms = ticks_to_ms(track_time, &tempo_map);
                note_changes.insert(ms, active_notes.clone());
            }
            midly::MidiMessage::NoteOff { key, .. } => {
                active_notes.retain(|&x| x != key.as_int());
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