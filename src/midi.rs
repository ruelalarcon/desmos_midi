use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use midly::{Smf, TrackEventKind};

pub fn process_midi(midi_path: &str) -> Result<(), Box<dyn Error>> {
    // Read and parse MIDI file
    let midi_data = fs::read(midi_path)?;
    let smf = Smf::parse(&midi_data)?;

    // Get timing info (ticks per quarter note)
    let ticks_per_quarter = match smf.header.timing {
        midly::Timing::Metrical(ticks) => ticks.as_int() as u32,
        _ => return Err("Unsupported timing format".into()),
    };

    // Track active notes and their changes
    let mut active_notes = Vec::new();
    let mut note_changes: HashMap<u64, Vec<u8>> = HashMap::new();
    let mut tempo: u32 = 500000; // Default tempo (120 BPM)

    // Process all tracks
    for track in smf.tracks.iter() {
        let mut track_time: u64 = 0;

        for event in track.iter() {
            track_time += u64::from(event.delta.as_int());

            match event.kind {
                TrackEventKind::Midi { message, .. } => {
                    match message {
                        midly::MidiMessage::NoteOn { key, vel } => {
                            if vel.as_int() > 0 {
                                // Add note and keep sorted
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
                TrackEventKind::Meta(meta_message) => {
                    if let midly::MetaMessage::Tempo(tempo_val) = meta_message {
                        tempo = tempo_val.as_int();
                    }
                }
                _ => {}
            }
        }
    }

    // Sort note changes by timestamp
    let mut sorted_changes: Vec<(u64, Vec<u8>)> = note_changes.into_iter().collect();
    sorted_changes.sort_by_key(|(time, _)| *time);

    // Create export folder
    let folder_name = create_export_folder(midi_path)?;

    // Export data.json
    let data_json: HashMap<String, Vec<i32>> = sorted_changes
        .iter()
        .map(|(time, notes)| {
            (time.to_string(),
             notes.iter().map(|&n| midi_note_to_relative(n)).collect())
        })
        .collect();

    let json_path = folder_name.join("data.json");
    let json_string = serde_json::to_string_pretty(&data_json)?;
    fs::write(json_path, json_string)?;

    // Export formulas.txt
    let formulas: Vec<String> = sorted_changes
        .iter()
        .map(|(_, notes)| {
            let relative_notes: Vec<i32> = notes.iter()
                .map(|&n| midi_note_to_relative(n))
                .collect();
            format!("A\\to\\left[{}\\right]",
                   relative_notes.iter()
                       .map(|n| n.to_string())
                       .collect::<Vec<String>>()
                       .join(","))
        })
        .collect();

    let formulas_path = folder_name.join("formulas.txt");
    let mut file = File::create(formulas_path)?;
    writeln!(file, "{}", formulas.join("\n"))?;

    Ok(())
}

fn ticks_to_ms(ticks: u64, tempo: u32, ticks_per_quarter: u32) -> u64 {
    // Convert MIDI ticks to milliseconds
    // tempo is in microseconds per quarter note
    // Formula: (ticks * tempo) / (ticks_per_quarter * 1000)
    (ticks as u128 * tempo as u128 / (ticks_per_quarter as u128 * 1000)) as u64
}

fn midi_note_to_relative(note: u8) -> i32 {
    // Convert MIDI note to relative position from Bb (58)
    // This matches the Python version's conversion
    (note as i32) - 58
}

fn create_export_folder(midi_path: &str) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let path = Path::new(midi_path);
    let folder_name = path.file_stem()
        .ok_or("Invalid MIDI filename")?
        .to_string_lossy()
        .into_owned();

    let folder_path = Path::new(&folder_name);
    fs::create_dir_all(folder_path)?;

    Ok(folder_path.to_path_buf())
}