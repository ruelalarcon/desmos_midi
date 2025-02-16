mod types;
mod parser;
mod timing;
mod export;

use std::error::Error;
use std::fs;

pub fn process_midi(midi_path: &str) -> Result<(), Box<dyn Error>> {
    let midi_data = fs::read(midi_path)?;
    let song = parser::parse_midi(&midi_data)?;
    export::export_song(&song, midi_path)?;
    Ok(())
}