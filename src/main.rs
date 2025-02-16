use std::env;
use std::path::Path;
use std::process;

mod midi;
mod keyboard;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <command> <path>", args[0]);
        eprintln!("Commands:");
        eprintln!("  play <song_folder>    - Play a song from the specified folder");
        eprintln!("  export <midi_file>    - Export a MIDI file to JSON and formulas");
        process::exit(1);
    }

    let command = &args[1];
    let path = &args[2];

    match command.as_str() {
        "play" => {
            if !Path::new(path).exists() {
                eprintln!("Error: Folder {} not found", path);
                process::exit(1);
            }
            match keyboard::SongPlayer::new().and_then(|mut player| player.play_song(path)) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error playing song: {}", e);
                    process::exit(1);
                }
            }
        }
        "export" => {
            if !Path::new(path).exists() {
                eprintln!("Error: File {} not found", path);
                process::exit(1);
            }
            if let Err(e) = midi::process_midi(path) {
                eprintln!("Error processing MIDI: {}", e);
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            process::exit(1);
        }
    }
}