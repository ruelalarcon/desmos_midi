/// Represents the data from a WAV file after reading
#[derive(Debug)]
pub struct WavData {
    /// Raw samples normalized to [-1, 1] range
    pub samples: Vec<f32>,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of audio channels
    pub channels: u16,
}

/// Configuration for harmonic analysis
#[derive(Debug)]
pub struct AnalysisConfig {
    /// Number of samples to analyze
    pub samples: usize,
    /// Start time in seconds
    pub start_time: f32,
    /// Base frequency for harmonic analysis (Hz)
    pub base_freq: f32,
    /// Number of harmonics to extract
    pub num_harmonics: usize,
    /// Boost factor for the output (multiplies the final amplitudes)
    pub boost: f32,
}

/// Errors that can occur during audio processing
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    /// IO errors when reading/writing files
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors during WAV file parsing
    #[error("WAV parsing error: {0}")]
    WavParse(String),

    /// Invalid parameter values
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// General processing errors
    #[error("Processing error: {0}")]
    ProcessingError(String),
}

impl AnalysisConfig {
    /// Validates the configuration against the provided WAV data
    ///
    /// # Arguments
    /// * `wav_data` - The WAV data to validate against
    ///
    /// # Returns
    /// * `Ok(())` if the configuration is valid
    /// * `Err(AudioError)` if the configuration is invalid
    pub fn validate(&self, wav_data: &WavData) -> Result<(), AudioError> {
        let total_samples = wav_data.samples.len() / wav_data.channels as usize;
        let start_sample = (self.start_time * wav_data.sample_rate as f32) as usize;
        let duration = total_samples as f32 / wav_data.sample_rate as f32;

        // Check if start time is valid
        if self.start_time >= duration {
            return Err(AudioError::InvalidParams(format!(
                "Start time ({:.2}s) exceeds audio duration ({:.2}s)",
                self.start_time, duration
            )));
        }

        // Check if we have enough samples
        if start_sample + self.samples > total_samples {
            let available_samples = total_samples - start_sample;
            return Err(AudioError::InvalidParams(format!(
                "Not enough samples available. Requested {} samples starting at {:.2}s, but only {} samples available. Try reducing the sample count or start time.",
                self.samples, self.start_time, available_samples
            )));
        }

        // Check Nyquist frequency
        let nyquist = wav_data.sample_rate as f32 / 2.0;
        let max_harmonics = (nyquist / self.base_freq).floor() as usize;
        if self.base_freq * self.num_harmonics as f32 > nyquist {
            return Err(AudioError::InvalidParams(format!(
                "With base frequency of {:.1}Hz, maximum number of harmonics possible is {} (limited by Nyquist frequency of {:.1}Hz)",
                self.base_freq, max_harmonics, nyquist
            )));
        }

        Ok(())
    }
}
