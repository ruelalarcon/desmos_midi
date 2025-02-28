use super::types::{AudioError, WavData};
use hound::{SampleFormat, WavReader};
use std::path::Path;

/// Reads and parses a WAV file, converting samples to normalized f32 values.
///
/// This function supports the following WAV formats:
/// - 32-bit float
/// - 16-bit integer
/// - 24-bit integer
/// - 32-bit integer
///
/// All integer formats are normalized to the [-1, 1] range.
///
/// # Arguments
/// * `path` - Path to the WAV file to read
///
/// # Returns
/// * `Result<WavData, AudioError>` - Parsed WAV data or an error
///
/// # Errors
/// * If the file cannot be read
/// * If the WAV format is unsupported
/// * If there's an error during sample conversion
pub fn read_wav_file(path: &Path) -> Result<WavData, AudioError> {
    let reader = WavReader::open(path).map_err(|e| AudioError::WavParse(e.to_string()))?;
    let spec = reader.spec();

    // Convert samples to f32, regardless of input format
    let samples: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Float, 32) => reader
            .into_samples::<f32>()
            .map(|s| s.map_err(|e| AudioError::WavParse(e.to_string())))
            .collect::<Result<Vec<f32>, AudioError>>()?,
        (SampleFormat::Int, 16) => reader
            .into_samples::<i16>()
            .map(|s| s.map_err(|e| AudioError::WavParse(e.to_string())))
            .map(|s| Ok(s? as f32 / 32768.0))
            .collect::<Result<Vec<f32>, AudioError>>()?,
        (SampleFormat::Int, 24) => reader
            .into_samples::<i32>()
            .map(|s| s.map_err(|e| AudioError::WavParse(e.to_string())))
            .map(|s| Ok(s? as f32 / 8388608.0))
            .collect::<Result<Vec<f32>, AudioError>>()?,
        (SampleFormat::Int, 32) => reader
            .into_samples::<i32>()
            .map(|s| s.map_err(|e| AudioError::WavParse(e.to_string())))
            .map(|s| Ok(s? as f32 / 2147483648.0))
            .collect::<Result<Vec<f32>, AudioError>>()?,
        _ => {
            return Err(AudioError::WavParse(format!(
                "Unsupported WAV format: {:?} {}-bit",
                spec.sample_format, spec.bits_per_sample
            )))
        }
    };

    Ok(WavData {
        samples,
        sample_rate: spec.sample_rate,
        channels: spec.channels,
    })
}
