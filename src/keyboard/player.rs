use std::error::Error;
use std::fs;
use std::io::Read;
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
        let timestamps = self.load_timing_data(folder_name)?;
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
        let update_interval = Duration::from_millis(100);
        let mut last_update = Instant::now();

        for i in 0..timestamps.len() - 1 {
            let next_timestamp = timestamps[i + 1];

            loop {
                if self.keyboard.is_key_pressed(&self.config.keyboard.stop_key) {
                    println!("\nPlayback stopped");
                    return Ok(());
                }

                let elapsed = start_time.elapsed().as_millis() as u64;

                if last_update.elapsed() >= update_interval {
                    print!("\rProgress: {} / {}",
                           format_time(elapsed),
                           format_time(total_duration));
                    std::io::Write::flush(&mut std::io::stdout())?;
                    last_update = Instant::now();
                }

                if elapsed >= next_timestamp {
                    break;
                }

                thread::sleep(Duration::from_millis(10));
            }

            if self.config.timing.note_delay_ms > 0 {
                thread::sleep(Duration::from_millis(self.config.timing.note_delay_ms as u64));
            }

            for key in &self.config.keyboard.next_sequence {
                self.keyboard.send_key(key);
            }
        }

        println!("\rProgress: {} / {}",
                 format_time(total_duration),
                 format_time(total_duration));

        Ok(())
    }

    fn load_timing_data(&self, folder_name: &str) -> Result<Vec<u64>, Box<dyn Error>> {
        let path = Path::new(folder_name).join("timing.bin");
        let mut file = fs::File::open(path)?;
        let mut timestamps = Vec::new();
        let mut buffer = [0u8; 8]; // u64 = 8 bytes

        while file.read_exact(&mut buffer).is_ok() {
            timestamps.push(u64::from_le_bytes(buffer));
        }

        Ok(timestamps)
    }
}

fn format_time(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}