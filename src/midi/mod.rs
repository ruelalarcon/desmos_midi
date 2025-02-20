mod types;
mod parser;
mod timing;

use std::error::Error;
use std::fs;

pub fn process_midi(midi_path: &str) -> Result<String, Box<dyn Error>> {
    let midi_data = fs::read(midi_path)?;
    let song = parser::parse_midi(&midi_data)?;
    Ok(song.to_piecewise_function())
}