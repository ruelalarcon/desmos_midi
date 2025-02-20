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

    /// Show MIDI channel information and exit
    #[arg(short, long)]
    info: bool,

    /// Soundfont files to use (in order of MIDI channels)
    #[arg(short, long = "soundfonts", value_delimiter = ' ', num_args = 1.., value_name = "FILE")]
    soundfonts: Vec<String>,
}

fn print_channel_info(song: &midi::ProcessedSong) {
    println!("MIDI Channel Information:");
    println!("------------------------");
    for channel in &song.channels {
        println!(
            "Channel {}: {} {}",
            channel.id + 1, // MIDI channels are 1-based in display
            if channel.is_drum { "[DRUMS] " } else { "" },
            midi::get_instrument_name(channel.instrument, channel.is_drum)
        );
    }
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.midi_file).exists() {
        eprintln!("Error: MIDI file {} not found", args.midi_file);
        process::exit(1);
    }

    // Process MIDI file
    let result = if args.info {
        midi::process_midi_info(&args.midi_file)
    } else {
        let soundfonts = if args.soundfonts.is_empty() {
            // If no soundfonts specified, use default.txt for all channels
            vec!["default.txt".to_string()]
        } else {
            args.soundfonts
        };
        midi::process_midi(&args.midi_file, soundfonts)
    };

    match result {
        Ok(song) => {
            if args.info {
                print_channel_info(&song);
            } else {
                let formula = song.to_piecewise_function();
                if args.copy {
                    // Copy to clipboard
                    if let Err(e) = ClipboardContext::new()
                        .and_then(|mut ctx| ctx.set_contents(formula))
                    {
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
        }
        Err(e) => {
            eprintln!("Error processing MIDI: {}", e);
            process::exit(1);
        }
    }
}