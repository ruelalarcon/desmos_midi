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
        // Special keys
        "right" => enigo.key_click(Key::RightArrow),
        "left" => enigo.key_click(Key::LeftArrow),
        "space" => enigo.key_click(Key::Space),
        "tab" => enigo.key_click(Key::Tab),
        "enter" => enigo.key_click(Key::Return),
        "up" => enigo.key_click(Key::UpArrow),
        "down" => enigo.key_click(Key::DownArrow),
        "escape" | "esc" => enigo.key_click(Key::Escape),

        // Function keys
        "f1" => enigo.key_click(Key::F1),
        "f2" => enigo.key_click(Key::F2),
        "f3" => enigo.key_click(Key::F3),
        "f4" => enigo.key_click(Key::F4),
        "f5" => enigo.key_click(Key::F5),
        "f6" => enigo.key_click(Key::F6),
        "f7" => enigo.key_click(Key::F7),
        "f8" => enigo.key_click(Key::F8),
        "f9" => enigo.key_click(Key::F9),
        "f10" => enigo.key_click(Key::F10),
        "f11" => enigo.key_click(Key::F11),
        "f12" => enigo.key_click(Key::F12),

        // Single letter keys
        "a" => enigo.key_click(Key::Layout('a')),
        "b" => enigo.key_click(Key::Layout('b')),
        "c" => enigo.key_click(Key::Layout('c')),
        "d" => enigo.key_click(Key::Layout('d')),
        "e" => enigo.key_click(Key::Layout('e')),
        "f" => enigo.key_click(Key::Layout('f')),
        "g" => enigo.key_click(Key::Layout('g')),
        "h" => enigo.key_click(Key::Layout('h')),
        "i" => enigo.key_click(Key::Layout('i')),
        "j" => enigo.key_click(Key::Layout('j')),
        "k" => enigo.key_click(Key::Layout('k')),
        "l" => enigo.key_click(Key::Layout('l')),
        "m" => enigo.key_click(Key::Layout('m')),
        "n" => enigo.key_click(Key::Layout('n')),
        "o" => enigo.key_click(Key::Layout('o')),
        "p" => enigo.key_click(Key::Layout('p')),
        "q" => enigo.key_click(Key::Layout('q')),
        "r" => enigo.key_click(Key::Layout('r')),
        "s" => enigo.key_click(Key::Layout('s')),
        "t" => enigo.key_click(Key::Layout('t')),
        "u" => enigo.key_click(Key::Layout('u')),
        "v" => enigo.key_click(Key::Layout('v')),
        "w" => enigo.key_click(Key::Layout('w')),
        "x" => enigo.key_click(Key::Layout('x')),
        "y" => enigo.key_click(Key::Layout('y')),
        "z" => enigo.key_click(Key::Layout('z')),

        _ => eprintln!("Unsupported key: {}", key),
    }
}

fn is_key_pressed(device_state: &DeviceState, key: &str) -> bool {
    let keys: Vec<Keycode> = device_state.get_keys();
    match key.to_lowercase().as_str() {
        // Special keys
        "esc" | "escape" => keys.contains(&Keycode::Escape),
        "tab" => keys.contains(&Keycode::Tab),
        "enter" | "return" => keys.contains(&Keycode::Enter),
        "space" => keys.contains(&Keycode::Space),
        "left" => keys.contains(&Keycode::Left),
        "right" => keys.contains(&Keycode::Right),
        "up" => keys.contains(&Keycode::Up),
        "down" => keys.contains(&Keycode::Down),

        // Function keys
        "f1" => keys.contains(&Keycode::F1),
        "f2" => keys.contains(&Keycode::F2),
        "f3" => keys.contains(&Keycode::F3),
        "f4" => keys.contains(&Keycode::F4),
        "f5" => keys.contains(&Keycode::F5),
        "f6" => keys.contains(&Keycode::F6),
        "f7" => keys.contains(&Keycode::F7),
        "f8" => keys.contains(&Keycode::F8),
        "f9" => keys.contains(&Keycode::F9),
        "f10" => keys.contains(&Keycode::F10),
        "f11" => keys.contains(&Keycode::F11),
        "f12" => keys.contains(&Keycode::F12),

        // Single letter keys
        "a" => keys.contains(&Keycode::A),
        "b" => keys.contains(&Keycode::B),
        "c" => keys.contains(&Keycode::C),
        "d" => keys.contains(&Keycode::D),
        "e" => keys.contains(&Keycode::E),
        "f" => keys.contains(&Keycode::F),
        "g" => keys.contains(&Keycode::G),
        "h" => keys.contains(&Keycode::H),
        "i" => keys.contains(&Keycode::I),
        "j" => keys.contains(&Keycode::J),
        "k" => keys.contains(&Keycode::K),
        "l" => keys.contains(&Keycode::L),
        "m" => keys.contains(&Keycode::M),
        "n" => keys.contains(&Keycode::N),
        "o" => keys.contains(&Keycode::O),
        "p" => keys.contains(&Keycode::P),
        "q" => keys.contains(&Keycode::Q),
        "r" => keys.contains(&Keycode::R),
        "s" => keys.contains(&Keycode::S),
        "t" => keys.contains(&Keycode::T),
        "u" => keys.contains(&Keycode::U),
        "v" => keys.contains(&Keycode::V),
        "w" => keys.contains(&Keycode::W),
        "x" => keys.contains(&Keycode::X),
        "y" => keys.contains(&Keycode::Y),
        "z" => keys.contains(&Keycode::Z),

        _ => {
            eprintln!("Unsupported key for detection: {}", key);
            false
        }
    }
}