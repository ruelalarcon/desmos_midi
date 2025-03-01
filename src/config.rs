use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Common configuration for both CLI and Web UI
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub common: CommonConfig,
    #[serde(default)]
    pub server: Option<ServerConfig>,
}

/// Common configuration settings
#[derive(Debug, Clone, Deserialize)]
pub struct CommonConfig {
    /// Directory where soundfonts are stored
    pub soundfonts_dir: String,
}

/// Server-specific configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub file_expiration_minutes: u64,
    pub file_refresh_threshold_minutes: u64,
    pub max_file_size_mb: u64,
    pub limits: AnalysisLimits,
}

/// Analysis limits for WAV processing
#[derive(Debug, Clone, Deserialize)]
pub struct AnalysisLimits {
    pub min_samples: usize,
    pub max_samples: usize,
    pub min_start_time: f32,
    pub max_start_time: f32,
    pub min_base_freq: f32,
    pub max_base_freq: f32,
    pub min_harmonics: usize,
    pub max_harmonics: usize,
    pub min_boost: f32,
    pub max_boost: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            common: CommonConfig {
                soundfonts_dir: "soundfonts".to_string(),
            },
            server: Some(ServerConfig::default()),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            file_expiration_minutes: 10,
            file_refresh_threshold_minutes: 5,
            max_file_size_mb: 80,
            limits: AnalysisLimits::default(),
        }
    }
}

impl Default for AnalysisLimits {
    fn default() -> Self {
        AnalysisLimits {
            min_samples: 64,
            max_samples: 65536,
            min_start_time: 0.0,
            max_start_time: 300.0,
            min_base_freq: 1.0,
            max_base_freq: 20000.0,
            min_harmonics: 1,
            max_harmonics: 256,
            min_boost: 0.5,
            max_boost: 2.0,
        }
    }
}

/// Load configuration from config.toml
pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Try to load from config.toml
    match File::open("config.toml") {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(toml::from_str(&contents)?)
        }
        Err(_) => {
            // If file doesn't exist, return default config
            Ok(Config::default())
        }
    }
}

/// Get the soundfonts directory path
pub fn get_soundfonts_dir() -> PathBuf {
    let config = load_config().unwrap_or_default();
    PathBuf::from(config.common.soundfonts_dir)
}

/// Ensure the soundfonts directory exists
pub fn ensure_soundfonts_dir() -> Result<PathBuf, std::io::Error> {
    let dir = get_soundfonts_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}
