use clap::{Parser, Subcommand};
use clipboard::{ClipboardContext, ClipboardProvider};
use std::io::{self, Write};
use std::path::Path;
use std::process;

mod midi;
use midi::{MidiError, MidiProcessor};

use desmos_midi::audio::{self, AnalysisConfig, AudioError};
use desmos_midi::config;

/// Desmos MIDI and Audio Analysis Tool
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert MIDI files to Desmos formulas
    Midi(MidiArgs),

    /// Analyze WAV files to create soundfonts
    Audio(AudioArgs),
}

/// Convert MIDI files to Desmos formulas
#[derive(Parser)]
struct MidiArgs {
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

/// Analyze WAV files to create soundfonts
#[derive(Parser)]
struct AudioArgs {
    /// Path to the input WAV file
    #[arg(required = true)]
    wav_file: String,

    /// Number of samples to analyze
    #[arg(long, default_value_t = 8192)]
    samples: usize,

    /// Position in audio to begin analysis (seconds)
    #[arg(long, default_value_t = 0.0)]
    start_time: f32,

    /// Fundamental frequency to analyze (Hz)
    #[arg(long, default_value_t = 440.0)]
    base_freq: f32,

    /// Number of harmonics to extract
    #[arg(long, default_value_t = 16)]
    harmonics: usize,

    /// Amplification factor for harmonics
    #[arg(long, default_value_t = 1.0)]
    boost: f32,

    /// Copy output to clipboard instead of console
    #[arg(short, long)]
    copy: bool,
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

fn run_midi_command(args: &MidiArgs) -> Result<(), MidiError> {
    // Check if MIDI file exists with a clear error message
    if !Path::new(&args.midi_file).exists() {
        return Err(MidiError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("MIDI file not found: {}", args.midi_file),
        )));
    }

    // Create MidiProcessor using the directory from config.toml
    let soundfonts_dir = config::get_soundfonts_dir();
    let processor =
        MidiProcessor::with_soundfont_dir(soundfonts_dir.to_str().unwrap_or("soundfonts"));

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

fn run_audio_command(args: &AudioArgs) -> Result<(), AudioError> {
    // Check if WAV file exists
    let wav_path = Path::new(&args.wav_file);
    if !wav_path.exists() {
        return Err(AudioError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("WAV file not found: {}", args.wav_file),
        )));
    }

    // Read WAV file and analyze without printing status messages
    let wav_data = audio::read_wav_file(wav_path)?;

    // Create analysis config
    let config = AnalysisConfig {
        samples: args.samples,
        start_time: args.start_time,
        base_freq: args.base_freq,
        num_harmonics: args.harmonics,
        boost: args.boost,
    };

    // Analyze harmonics
    let harmonics = audio::analyze_harmonics(&wav_data, &config)?;

    // Format the harmonics as a comma-separated string
    let output = harmonics
        .iter()
        .map(|h| h.to_string())
        .collect::<Vec<String>>()
        .join(",");

    if args.copy {
        // Copy to clipboard
        ClipboardContext::new()
            .map_err(|e| AudioError::ProcessingError(e.to_string()))?
            .set_contents(output)
            .map_err(|e| AudioError::ProcessingError(e.to_string()))?;
        println!("Successfully copied soundfont to clipboard!");
    } else {
        // Output directly to console, just the weights
        io::stdout().write_all(output.as_bytes())?;
    }

    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Midi(args) => run_midi_command(args)?,
        Commands::Audio(args) => run_audio_command(args)?,
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("\nERROR: {}\n", err);
            match err.downcast_ref::<MidiError>() {
                Some(MidiError::Io(ref io_err)) if io_err.kind() == io::ErrorKind::NotFound => {
                    eprintln!("Please check that:");
                    eprintln!("1. The file path is correct");
                    eprintln!("2. The file exists");
                    eprintln!("3. You have permission to read the file");
                }
                Some(MidiError::InvalidSoundfont(ref _msg)) => {
                    eprintln!("Make sure the file exists in the soundfonts directory!");
                }
                _ => {}
            }
            process::exit(1);
        }
    }
}
