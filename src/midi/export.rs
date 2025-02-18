use std::error::Error;
use std::fs::File;
use std::io::Write;
use clipboard::{ClipboardContext, ClipboardProvider};
use super::types::ProcessedSong;

pub fn export_song(song: &ProcessedSong, output_path: Option<&str>) -> Result<(), Box<dyn Error>> {
    let formula = song.to_piecewise_function();

    match output_path {
        Some(path) => {
            // Export to file
            let mut file = File::create(path)?;
            write!(file, "{}", formula)?;
            println!("Exported to file: {}", path);
        }
        None => {
            // Copy to clipboard
            let mut ctx = ClipboardContext::new()?;
            ctx.set_contents(formula)?;
            println!("Copied to clipboard!");
        }
    }

    Ok(())
}