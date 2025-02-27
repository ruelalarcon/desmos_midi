mod parser;
mod soundfonts;
mod timing;
mod types;

pub use soundfonts::{get_instrument_name, parse_soundfont_file, soundfont_exists};
use std::fs;
use std::path::Path;
pub use types::{MidiError, ProcessedSong};

/// MIDI processor for handling MIDI file processing
pub struct MidiProcessor {
    soundfont_dir: Option<String>,
}

impl MidiProcessor {
    /// Creates a new MidiProcessor with default settings.
    pub fn new() -> Self {
        Self {
            soundfont_dir: None,
        }
    }

    /// Creates a new MidiProcessor with a custom soundfont directory.
    pub fn with_soundfont_dir<P: Into<String>>(soundfont_dir: P) -> Self {
        Self {
            soundfont_dir: Some(soundfont_dir.into()),
        }
    }

    /// Parses a MIDI file and returns channel information.
    ///
    /// This is a lightweight parse that only extracts channel and instrument information,
    /// without processing note events.
    ///
    /// # Arguments
    /// * `midi_path` - Path to the MIDI file
    ///
    /// # Returns
    /// * `ProcessedSong` - Song information with only channel data (no notes)
    ///
    /// # Errors
    /// * If the file cannot be read
    /// * If the MIDI file is invalid
    pub fn process_info(&self, midi_path: &str) -> Result<ProcessedSong, MidiError> {
        let midi_data = fs::read(midi_path)?;
        parser::parse_midi(&midi_data, true)
    }

    /// Processes a MIDI file with soundfont information.
    ///
    /// This function:
    /// 1. Reads the MIDI file
    /// 2. Extracts channel information
    /// 3. Validates the number of soundfonts matches channels
    /// 4. Maps channels to soundfonts
    /// 5. Processes note events with soundfont assignments
    ///
    /// # Arguments
    /// * `midi_path` - Path to the MIDI file
    /// * `soundfont_files` - Vector of soundfont filenames to use
    ///                     If only one is provided, it's used for all channels
    ///
    /// # Returns
    /// * `ProcessedSong` - Fully processed song with notes and soundfonts
    ///
    /// # Errors
    /// * If the file cannot be read
    /// * If the MIDI file is invalid
    /// * If the number of soundfonts doesn't match the number of channels
    /// * If any soundfont file cannot be read or is invalid
    pub fn process_with_soundfonts(
        &self,
        midi_path: &str,
        mut soundfont_files: Vec<String>,
    ) -> Result<ProcessedSong, MidiError> {
        let midi_data = fs::read(midi_path)?;

        // First parse MIDI to get channel info
        let info_song = parser::parse_midi(&midi_data, true)?;
        let channel_count = info_song.channels.len();

        // Validate soundfont count matches channel count
        if soundfont_files.len() == 1 {
            // If only one soundfont provided, duplicate it for all channels
            let default_font = soundfont_files[0].clone();
            soundfont_files = vec![default_font; channel_count];
        } else if soundfont_files.len() < channel_count {
            return Err(MidiError::SoundfontMismatch(format!(
                "Not enough soundfonts provided. Need {} for channels, got {}",
                channel_count,
                soundfont_files.len()
            )));
        } else if soundfont_files.len() > channel_count {
            return Err(MidiError::SoundfontMismatch(format!(
                "Too many soundfonts provided. Need {} for channels, got {}",
                channel_count,
                soundfont_files.len()
            )));
        }

        // Create a mapping of channel ID to soundfont index
        let mut channel_to_index = vec![None; 16]; // MIDI has 16 possible channels
        let mut active_soundfonts = Vec::with_capacity(channel_count);

        // Load all soundfonts and create channel mapping
        let soundfont_dir = self.soundfont_dir.as_ref().map(Path::new);

        for (_idx, (channel, soundfont_file)) in info_song
            .channels
            .iter()
            .zip(soundfont_files.iter())
            .enumerate()
        {
            if let Some(soundfont) = parse_soundfont_file(soundfont_file, soundfont_dir)? {
                channel_to_index[channel.id as usize] = Some(active_soundfonts.len());
                active_soundfonts.push(soundfont);
            }
        }

        // Now parse MIDI with soundfonts and channel mapping
        parser::parse_midi_with_soundfonts(&midi_data, active_soundfonts, channel_to_index)
    }

    /// Verifies that all soundfont files exist.
    ///
    /// # Arguments
    /// * `soundfont_files` - Vector of soundfont filenames to check
    ///
    /// # Returns
    /// * `Result<(), MidiError>` - Ok if all files exist, Err otherwise
    pub fn verify_soundfonts(&self, soundfont_files: &[String]) -> Result<(), MidiError> {
        let soundfont_dir = self.soundfont_dir.as_ref().map(Path::new);
        for file in soundfont_files {
            if !soundfont_exists(file, soundfont_dir) {
                return Err(MidiError::InvalidSoundfont(format!(
                    "Soundfont file not found: {}",
                    file
                )));
            }
        }
        Ok(())
    }
}

// Legacy functions for backward compatibility
/// Parses a MIDI file and returns channel information.
#[allow(dead_code)]
pub fn process_midi_info(midi_path: &str) -> Result<ProcessedSong, MidiError> {
    MidiProcessor::new().process_info(midi_path)
}

/// Processes a MIDI file with soundfont information.
#[allow(dead_code)]
pub fn process_midi(
    midi_path: &str,
    soundfont_files: Vec<String>,
) -> Result<ProcessedSong, MidiError> {
    MidiProcessor::new().process_with_soundfonts(midi_path, soundfont_files)
}
