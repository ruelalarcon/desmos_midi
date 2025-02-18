use std::env;
use std::path::Path;
use std::process;

mod midi;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <midi_file>", args[0]);
        eprintln!("Converts a MIDI file to a Desmos piecewise function");
        process::exit(1);
    }

    let midi_path = &args[1];

    if !Path::new(midi_path).exists() {
        eprintln!("Error: File {} not found", midi_path);
        process::exit(1);
    }

    if let Err(e) = midi::process_midi(midi_path) {
        eprintln!("Error processing MIDI: {}", e);
        process::exit(1);
    }

    println!("Successfully exported piecewise function to {}.txt", 
             Path::new(midi_path)
                 .file_stem()
                 .unwrap()
                 .to_string_lossy());
}