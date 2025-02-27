use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::io::{self, Write};
use std::path::Path;
use std::process;

mod midi;
use midi::{MidiError, MidiProcessor};

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

    /// Custom directory for soundfont files
    #[arg(long = "soundfont-dir")]
    soundfont_dir: Option<String>,
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

/// Custom conversion from clipboard error to MidiError
fn clipboard_error<E: std::fmt::Display>(err: E) -> MidiError {
    MidiError::ClipboardError(err.to_string())
}

fn run() -> Result<(), MidiError> {
    let args = Args::parse();

    // Check if MIDI file exists with a clear error message
    if !Path::new(&args.midi_file).exists() {
        return Err(MidiError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("MIDI file not found: {}", args.midi_file),
        )));
    }

    // Create MidiProcessor
    let processor = match &args.soundfont_dir {
        Some(dir) => MidiProcessor::with_soundfont_dir(dir),
        None => MidiProcessor::new(),
    };

    // Process MIDI file
    let song = if args.info {
        processor.process_info(&args.midi_file)?
    } else {
        let soundfonts = if args.soundfonts.is_empty() {
            // First get channel info to identify drum channels
            let info = processor.process_info(&args.midi_file)?;

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
        } else {
            // Process each soundfont name to ensure .txt extension
            let soundfonts: Vec<String> = args
                .soundfonts
                .iter()
                .map(|s| process_soundfont_name(s))
                .collect();

            // Verify all soundfonts exist before proceeding
            processor.verify_soundfonts(&soundfonts)?;
            soundfonts
        };

        processor.process_with_soundfonts(&args.midi_file, soundfonts)?
    };

    if args.info {
        print_channel_info(&song);
    } else {
        let formula = song.to_piecewise_function();
        if args.copy {
            // Copy to clipboard
            ClipboardContext::new()
                .map_err(clipboard_error)?
                .set_contents(formula)
                .map_err(clipboard_error)?;
            println!("Successfully copied to clipboard!");
        } else {
            // Output to console
            io::stdout().write_all(formula.as_bytes())?;
        }
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("\nERROR: {}\n", err);
            match err {
                MidiError::Io(ref io_err) if io_err.kind() == io::ErrorKind::NotFound => {
                    eprintln!("Please check that:");
                    eprintln!("1. The file path is correct");
                    eprintln!("2. The file exists");
                    eprintln!("3. You have permission to read the file");
                }
                MidiError::InvalidSoundfont(ref _msg) => {
                    eprintln!("Make sure the file exists in the soundfonts directory!");
                }
                _ => {}
            }
            process::exit(1);
        }
    }
}
