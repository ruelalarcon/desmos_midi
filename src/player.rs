use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::*;
use serde_json;

use crate::config::Config;

pub fn play_song(folder_name: &str) -> Result<(), Box<dyn Error>> {
    // Load config and song data
    let config = Config::load()?;
    let song_data = load_song_data(folder_name)?;

    // Convert timestamps to integers and sort them
    let mut timestamps: Vec<i64> = song_data.keys()
        .map(|k| k.parse::<i64>())
        .collect::<Result<Vec<i64>, _>>()?;
    timestamps.sort_unstable();

    let total_duration = *timestamps.last().unwrap();

    println!("Playing {} in {} seconds...", folder_name, config.timing.initial_delay);
    println!("Press '{}' to stop playback", config.keyboard.stop_key);
    println!("Total duration: {}", format_time(total_duration));

    thread::sleep(Duration::from_secs_f64(config.timing.initial_delay));

    let mut enigo = Enigo::new();
    let device_state = DeviceState::new();
    let start_time = Instant::now();

    // Send initial key sequence
    for key in &config.keyboard.start_sequence {
        send_key(&mut enigo, key);
    }

    for i in 0..timestamps.len() - 1 {
        // Check for stop key
        if is_key_pressed(&device_state, &config.keyboard.stop_key) {
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
        if config.timing.note_delay_ms > 0 {
            thread::sleep(Duration::from_millis(config.timing.note_delay_ms as u64));
        }

        // Send next key sequence
        for key in &config.keyboard.next_sequence {
            send_key(&mut enigo, key);
        }
    }

    println!("\rProgress: {} / {}",
             format_time(total_duration),
             format_time(total_duration));

    Ok(())
}

fn load_song_data(folder_name: &str) -> Result<HashMap<String, Vec<i64>>, Box<dyn Error>> {
    let path = Path::new(folder_name).join("data.json");
    let data = fs::read_to_string(path)?;
    let song_data: HashMap<String, Vec<i64>> = serde_json::from_str(&data)?;
    Ok(song_data)
}

fn format_time(ms: i64) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

fn send_key(enigo: &mut Enigo, key: &str) {
    match key.to_lowercase().as_str() {
        "right" => enigo.key_click(Key::RightArrow),
        "left" => enigo.key_click(Key::LeftArrow),
        "space" => enigo.key_click(Key::Space),
        "tab" => enigo.key_click(Key::Tab),
        "enter" => enigo.key_click(Key::Return),
        "up" => enigo.key_click(Key::UpArrow),
        "down" => enigo.key_click(Key::DownArrow),
        "escape" | "esc" => enigo.key_click(Key::Escape),
        // Add more key mappings as needed
        _ => eprintln!("Unsupported key: {}", key),
    }
}

fn is_key_pressed(device_state: &DeviceState, key: &str) -> bool {
    let keys: Vec<Keycode> = device_state.get_keys();
    match key.to_lowercase().as_str() {
        "esc" | "escape" => keys.contains(&Keycode::Escape),
        "tab" => keys.contains(&Keycode::Tab),
        "enter" | "return" => keys.contains(&Keycode::Enter),
        "space" => keys.contains(&Keycode::Space),
        "left" => keys.contains(&Keycode::Left),
        "right" => keys.contains(&Keycode::Right),
        "up" => keys.contains(&Keycode::Up),
        "down" => keys.contains(&Keycode::Down),
        _ => {
            eprintln!("Unsupported key for detection: {}", key);
            false
        }
    }
}