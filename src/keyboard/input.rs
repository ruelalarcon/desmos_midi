use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{Enigo, Key, KeyboardControllable};

pub struct KeyboardController {
    enigo: Enigo,
    device_state: DeviceState,
}

impl KeyboardController {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(),
            device_state: DeviceState::new(),
        }
    }

    pub fn send_key(&mut self, key: &str) {
        match key.to_lowercase().as_str() {
            // Special keys
            "right" => self.enigo.key_click(Key::RightArrow),
            "left" => self.enigo.key_click(Key::LeftArrow),
            "space" => self.enigo.key_click(Key::Space),
            "tab" => self.enigo.key_click(Key::Tab),
            "enter" => self.enigo.key_click(Key::Return),
            "up" => self.enigo.key_click(Key::UpArrow),
            "down" => self.enigo.key_click(Key::DownArrow),
            "escape" | "esc" => self.enigo.key_click(Key::Escape),

            // Function keys
            "f1" => self.enigo.key_click(Key::F1),
            "f2" => self.enigo.key_click(Key::F2),
            "f3" => self.enigo.key_click(Key::F3),
            "f4" => self.enigo.key_click(Key::F4),
            "f5" => self.enigo.key_click(Key::F5),
            "f6" => self.enigo.key_click(Key::F6),
            "f7" => self.enigo.key_click(Key::F7),
            "f8" => self.enigo.key_click(Key::F8),
            "f9" => self.enigo.key_click(Key::F9),
            "f10" => self.enigo.key_click(Key::F10),
            "f11" => self.enigo.key_click(Key::F11),
            "f12" => self.enigo.key_click(Key::F12),

            // Single letter keys
            key if key.len() == 1 => {
                if let Some(c) = key.chars().next() {
                    self.enigo.key_click(Key::Layout(c));
                }
            }
            _ => eprintln!("Unsupported key: {}", key),
        }
    }

    pub fn is_key_pressed(&self, key: &str) -> bool {
        let keys: Vec<Keycode> = self.device_state.get_keys();
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
            key if key.len() == 1 => {
                if let Some(c) = key.chars().next() {
                    keys.contains(&match c {
                        'a' => Keycode::A,
                        'b' => Keycode::B,
                        'c' => Keycode::C,
                        'd' => Keycode::D,
                        'e' => Keycode::E,
                        'f' => Keycode::F,
                        'g' => Keycode::G,
                        'h' => Keycode::H,
                        'i' => Keycode::I,
                        'j' => Keycode::J,
                        'k' => Keycode::K,
                        'l' => Keycode::L,
                        'm' => Keycode::M,
                        'n' => Keycode::N,
                        'o' => Keycode::O,
                        'p' => Keycode::P,
                        'q' => Keycode::Q,
                        'r' => Keycode::R,
                        's' => Keycode::S,
                        't' => Keycode::T,
                        'u' => Keycode::U,
                        'v' => Keycode::V,
                        'w' => Keycode::W,
                        'x' => Keycode::X,
                        'y' => Keycode::Y,
                        'z' => Keycode::Z,
                        _ => return false,
                    })
                } else {
                    false
                }
            }
            _ => {
                eprintln!("Unsupported key for detection: {}", key);
                false
            }
        }
    }
}