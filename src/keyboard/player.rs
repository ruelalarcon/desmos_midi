use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use super::input::KeyboardController;
use super::types::Config;

pub struct SongPlayer {
    keyboard: KeyboardController,
    config: Config,
}

impl SongPlayer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            keyboard: KeyboardController::new(),
            config: Config::load()?,
        })
    }

    pub fn play_song(&mut self, folder_name: &str) -> Result<(), Box<dyn Error>> {
        let song_data = self.load_song_data(folder_name)?;

        // Convert timestamps to integers and sort them
        let mut timestamps: Vec<i64> = song_data.keys()
            .map(|k| k.parse::<i64>())
            .collect::<Result<Vec<i64>, _>>()?;
        timestamps.sort_unstable();

        let total_duration = *timestamps.last().unwrap();

        println!("Playing {} in {} seconds...", folder_name, self.config.timing.initial_delay);
        println!("Press '{}' to stop playback", self.config.keyboard.stop_key);
        println!("Total duration: {}", format_time(total_duration));

        thread::sleep(Duration::from_secs_f64(self.config.timing.initial_delay));

        // Send initial key sequence
        for key in &self.config.keyboard.start_sequence {
            self.keyboard.send_key(key);
        }

        let start_time = Instant::now();

        for i in 0..timestamps.len() - 1 {
            // Check for stop key
            if self.keyboard.is_key_pressed(&self.config.keyboard.stop_key) {
                println!("\nPlayback stopped");
                return Ok(());
            }

            let elapsed = start_time.elapsed().as_millis() as i64;
            let next_timestamp = timestamps[i + 1];

            // Show progress
            print!("\rProgress: {} / {}",
                   format_time(elapsed),
                   format_time(total_duration));

            // Wait until it's time for the next note
            let wait_time = next_timestamp - elapsed;
            if wait_time > 0 {
                thread::sleep(Duration::from_millis(wait_time as u64));
            }

            // Add configured delay
            if self.config.timing.note_delay_ms > 0 {
                thread::sleep(Duration::from_millis(self.config.timing.note_delay_ms as u64));
            }

            // Send next key sequence
            for key in &self.config.keyboard.next_sequence {
                self.keyboard.send_key(key);
            }
        }

        println!("\rProgress: {} / {}",
                 format_time(total_duration),
                 format_time(total_duration));

        Ok(())
    }

    fn load_song_data(&self, folder_name: &str) -> Result<HashMap<String, Vec<i64>>, Box<dyn Error>> {
        let path = Path::new(folder_name).join("data.json");
        let data = fs::read_to_string(path)?;
        let song_data: HashMap<String, Vec<i64>> = serde_json::from_str(&data)?;
        Ok(song_data)
    }
}

fn format_time(ms: i64) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}