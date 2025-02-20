use std::path::Path;
use std::process;
use std::io::{self, Write};
use clipboard::{ClipboardContext, ClipboardProvider};
use clap::Parser;

mod midi;

/// Convert MIDI files to Desmos formulas
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input MIDI file
    #[arg(required = true)]
    midi_file: String,

    /// Copy output to clipboard instead of console
    #[arg(short, long)]
    copy: bool,
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.midi_file).exists() {
        eprintln!("Error: MIDI file {} not found", args.midi_file);
        process::exit(1);
    }

    match midi::process_midi(&args.midi_file) {
        Ok(formula) => {
            if args.copy {
                // Copy to clipboard
                if let Err(e) = ClipboardContext::new()
                    .and_then(|mut ctx| ctx.set_contents(formula)) {
                    eprintln!("Failed to copy to clipboard: {}", e);
                    process::exit(1);
                }
                println!("Copied to clipboard!");
            } else {
                // Output to console
                if let Err(e) = io::stdout().write_all(formula.as_bytes()) {
                    eprintln!("Failed to write to console: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error processing MIDI: {}", e);
            process::exit(1);
        }
    }
}