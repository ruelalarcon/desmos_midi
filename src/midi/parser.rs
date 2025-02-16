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

    let mut active_notes = Vec::new();
    let mut note_changes: HashMap<Timestamp, Vec<MidiNote>> = HashMap::new();
    let tempo: u32 = 500000; // Default tempo (120 BPM)

    process_tracks(&smf.tracks, &mut active_notes, &mut note_changes, ticks_per_quarter, tempo)?;

    let mut events: Vec<NoteEvent> = note_changes
        .into_iter()
        .map(|(timestamp, notes)| NoteEvent { timestamp, notes })
        .collect();
    events.sort_by_key(|event| event.timestamp);

    Ok(ProcessedSong { note_changes: events })
}

fn process_tracks(
    tracks: &[midly::Track],
    active_notes: &mut Vec<MidiNote>,
    note_changes: &mut HashMap<Timestamp, Vec<MidiNote>>,
    ticks_per_quarter: u32,
    mut tempo: u32,
) -> Result<(), Box<dyn Error>> {
    for track in tracks {
        let mut track_time: u64 = 0;

        for event in track.iter() {
            track_time += u64::from(event.delta.as_int());

            match event.kind {
                TrackEventKind::Midi { message, .. } => {
                    process_midi_message(
                        message,
                        active_notes,
                        note_changes,
                        track_time,
                        tempo,
                        ticks_per_quarter,
                    );
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
    Ok(())
}

fn process_midi_message(
    message: midly::MidiMessage,
    active_notes: &mut Vec<MidiNote>,
    note_changes: &mut HashMap<Timestamp, Vec<MidiNote>>,
    track_time: u64,
    tempo: u32,
    ticks_per_quarter: u32,
) {
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