use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub keyboard: KeyboardConfig,
    pub timing: TimingConfig,
}

#[derive(Deserialize)]
pub struct KeyboardConfig {
    pub stop_key: String,
    pub start_sequence: Vec<String>,
    pub next_sequence: Vec<String>,
}

#[derive(Deserialize)]
pub struct TimingConfig {
    pub initial_delay: f64,
    pub note_delay_ms: i64,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}