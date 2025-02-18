use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use super::types::ProcessedSong;

pub fn export_song(song: &ProcessedSong, midi_path: &str) -> Result<(), Box<dyn Error>> {
    let output_path = create_output_path(midi_path)?;
    let mut file = File::create(output_path)?;
    write!(file, "{}", song.to_piecewise_function())?;
    Ok(())
}

fn create_output_path(midi_path: &str) -> Result<String, Box<dyn Error>> {
    let path = Path::new(midi_path);
    let stem = path.file_stem()
        .ok_or("Invalid MIDI filename")?
        .to_string_lossy()
        .into_owned();
    
    Ok(format!("{}.txt", stem))
}