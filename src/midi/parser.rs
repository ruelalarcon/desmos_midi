use std::error::Error;
use std::collections::HashMap;
use midly::{Smf, TrackEventKind};
use super::types::{Timestamp, MidiNote, NoteEvent, ProcessedSong};
use super::timing::ticks_to_ms;

pub fn parse_midi(midi_data: &[u8]) -> Result<ProcessedSong, Box<dyn Error>> {
    let smf = Smf::parse(midi_data)?;

    let ticks_per_quarter = match smf.header.timing {
        midly::Timing::Metrical(ticks) => ticks.as_int() as u32,
        _ => return Err("Unsupported timing format".into()),
    };

    // Collect all events from all tracks with their absolute timestamps
    let mut all_events = Vec::new();
    let mut tempo: u32 = 500000; // Default tempo (120 BPM)

    for track in smf.tracks.iter() {
        let mut track_time: u64 = 0;
        for event in track {
            track_time += u64::from(event.delta.as_int());
            match event.kind {
                TrackEventKind::Midi { message, .. } => {
                    all_events.push((track_time, message));
                }
                TrackEventKind::Meta(meta_message) => {
                    if let midly::MetaMessage::Tempo(tempo_val) = meta_message {
                        tempo = tempo_val.as_int();
                    }
                }
                _ => {}
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
                let ms = ticks_to_ms(track_time, tempo, ticks_per_quarter);
                note_changes.insert(ms, active_notes.clone());
            }
            midly::MidiMessage::NoteOff { key, .. } => {
                active_notes.retain(|&x| x != key.as_int());
                let ms = ticks_to_ms(track_time, tempo, ticks_per_quarter);
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