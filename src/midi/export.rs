use std::error::Error;
use std::fs::{self, File};
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use super::types::ProcessedSong;

pub fn export_song(song: &ProcessedSong, midi_path: &str) -> Result<(), Box<dyn Error>> {
    let folder_name = create_export_folder(midi_path)?;
    export_timing(song, &folder_name)?;
    export_formulas(song, &folder_name)?;
    Ok(())
}

fn export_timing(song: &ProcessedSong, folder: &Path) -> Result<(), Box<dyn Error>> {
    let timing_path = folder.join("timing.bin");
    let file = File::create(timing_path)?;
    let mut writer = BufWriter::new(file);

    // Write timestamps as binary data
    for timestamp in song.to_timing_data() {
        writer.write_all(&timestamp.to_le_bytes())?;
    }

    Ok(())
}

fn export_formulas(song: &ProcessedSong, folder: &Path) -> Result<(), Box<dyn Error>> {
    let formulas_path = folder.join("formulas.txt");
    let mut file = File::create(formulas_path)?;
    writeln!(file, "{}", song.to_formulas().join("\n"))?;
    Ok(())
}

fn create_export_folder(midi_path: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = Path::new(midi_path);
    let folder_name = path.file_stem()
        .ok_or("Invalid MIDI filename")?
        .to_string_lossy()
        .into_owned();

    let folder_path = Path::new(&folder_name);
    fs::create_dir_all(folder_path)?;

    Ok(folder_path.to_path_buf())
}