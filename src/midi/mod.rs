mod types;
mod parser;
mod timing;
mod soundfonts;

use std::error::Error;
use std::fs;
pub use types::{ProcessedSong, Channel};
pub use soundfonts::{parse_soundfont_file, get_instrument_name};

pub fn process_midi_info(midi_path: &str) -> Result<ProcessedSong, Box<dyn Error>> {
    let midi_data = fs::read(midi_path)?;
    parser::parse_midi(&midi_data, true)
}

pub fn process_midi(midi_path: &str, mut soundfont_files: Vec<String>) -> Result<ProcessedSong, Box<dyn Error>> {
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
        return Err(format!("Not enough soundfonts provided. Need {} for channels, got {}",
            channel_count, soundfont_files.len()).into());
    } else if soundfont_files.len() > channel_count {
        return Err(format!("Too many soundfonts provided. Need {} for channels, got {}",
            channel_count, soundfont_files.len()).into());
    }

    // Create a mapping of channel ID to soundfont index
    let mut channel_to_index = vec![0; 16]; // MIDI has 16 possible channels

    // Map channels to soundfonts in the order they appear in the MIDI file
    for (idx, channel) in info_song.channels.iter().enumerate() {
        channel_to_index[channel.id as usize] = idx;
    }

    // Load all soundfonts in order they were provided
    let mut soundfonts = Vec::with_capacity(channel_count);
    for i in 0..channel_count {
        soundfonts.push(parse_soundfont_file(&soundfont_files[i])?);
    }

    // Now parse MIDI with soundfonts and channel mapping
    parser::parse_midi_with_soundfonts(&midi_data, soundfonts, channel_to_index)
}