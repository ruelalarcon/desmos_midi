use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::io::{self, Write};
use std::path::Path;
use std::process;

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

/// Process a soundfont filename to ensure it has a .txt extension
fn process_soundfont_name(name: &str) -> String {
    if name == "-" {
        name.to_string()
    } else if !name.ends_with(".txt") {
        format!("{}.txt", name)
    } else {
        name.to_string()
    }
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

fn verify_soundfont_exists(name: &str) -> Result<(), String> {
    if name == "-" {
        return Ok(());
    }
    let path = Path::new("soundfonts").join(name);
    if !path.exists() {
        return Err(format!("ERROR: Soundfont file not found: {}\nMake sure the file exists in the 'soundfonts' directory!", name));
    }
    Ok(())
}

fn main() {
    let args = Args::parse();

    // Check if MIDI file exists with a clear error message
    if !Path::new(&args.midi_file).exists() {
        eprintln!("\nERROR: MIDI file not found: {}\n", args.midi_file);
        eprintln!("Please check that:");
        eprintln!("1. The file path is correct");
        eprintln!("2. The file exists");
        eprintln!("3. You have permission to read the file\n");
        process::exit(1);
    }

    // Process MIDI file
    let result = if args.info {
        midi::process_midi_info(&args.midi_file)
    } else {
        let soundfonts = if args.soundfonts.is_empty() {
            // First get channel info to identify drum channels
            match midi::process_midi_info(&args.midi_file) {
                Ok(info) => {
                    // Create soundfont list with "-" for drum channels and default.txt for others
                    info.channels
                        .iter()
                        .map(|ch| {
                            if ch.is_drum {
                                "-".to_string()
                            } else {
                                "default.txt".to_string()
                            }
                        })
                        .collect()
                }
                Err(e) => {
                    eprintln!("\nERROR: Failed to read MIDI file: {}\n", e);
                    process::exit(1);
                }
            }
        } else {
            // Process each soundfont name to ensure .txt extension
            let soundfonts: Vec<String> = args.soundfonts.iter().map(|s| process_soundfont_name(s)).collect();

            // Verify all soundfonts exist before proceeding
            for soundfont in &soundfonts {
                if let Err(e) = verify_soundfont_exists(soundfont) {
                    eprintln!("\n{}\n", e);
                    process::exit(1);
                }
            }
            soundfonts
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
                    if let Err(e) =
                        ClipboardContext::new().and_then(|mut ctx| ctx.set_contents(formula))
                    {
                        eprintln!("\nERROR: Failed to copy to clipboard: {}\n", e);
                        process::exit(1);
                    }
                    println!("Successfully copied to clipboard!");
                } else {
                    // Output to console
                    if let Err(e) = io::stdout().write_all(formula.as_bytes()) {
                        eprintln!("\nERROR: Failed to write to console: {}\n", e);
                        process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("\nERROR: Failed to process MIDI file: {}\n", e);
            process::exit(1);
        }
    }
}
