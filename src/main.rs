use std::env;
use std::path::Path;
use std::process;

mod midi;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <midi_file> [output_file]", args[0]);
        eprintln!("If output_file is not specified, the formula will be copied to clipboard");
        process::exit(1);
    }

    let midi_path = &args[1];
    let output_path = args.get(2);

    if !Path::new(midi_path).exists() {
        eprintln!("Error: MIDI file {} not found", midi_path);
        process::exit(1);
    }

    if let Err(e) = midi::process_midi(midi_path, output_path.map(|s| s.as_str())) {
        eprintln!("Error processing MIDI: {}", e);
        process::exit(1);
    }
}