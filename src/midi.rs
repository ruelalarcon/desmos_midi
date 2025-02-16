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

    // Track active notes and their changes
    let mut active_notes = Vec::new();
    let mut note_changes: HashMap<u64, Vec<u8>> = HashMap::new();

    let mut current_time: u64 = 0;

    // Process all tracks
    for track in smf.tracks.iter() {
        for event in track.iter() {
            current_time += event.delta.as_int() as u64;

            match event.kind {
                TrackEventKind::Midi { message, .. } => {
                    match message {
                        midly::MidiMessage::NoteOn { key, vel } => {
                            if vel.as_int() > 0 {
                                active_notes.push(key.as_int());
                            } else {
                                active_notes.retain(|&x| x != key.as_int());
                            }
                            note_changes.insert(current_time, active_notes.clone());
                        }
                        midly::MidiMessage::NoteOff { key, .. } => {
                            active_notes.retain(|&x| x != key.as_int());
                            note_changes.insert(current_time, active_notes.clone());
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // Create export folder
    let folder_name = create_export_folder(midi_path)?;

    // Export data.json
    let data_json: HashMap<String, Vec<i32>> = note_changes
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
    let formulas: Vec<String> = note_changes
        .values()
        .map(|notes| {
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

fn midi_note_to_relative(note: u8) -> i32 {
    (note as i32) - 58 // Bb as root note (0)
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