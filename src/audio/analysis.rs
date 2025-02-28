use super::types::{AnalysisConfig, AudioError, WavData};
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;

/// Analyzes a WAV file to extract harmonic content.
///
/// This function performs the following steps:
/// 1. Validates the analysis configuration
/// 2. Extracts mono samples from the WAV data
/// 3. Applies a Hann window to the samples
/// 4. Performs FFT analysis
/// 5. Extracts and normalizes harmonic weights
///
/// # Arguments
/// * `wav_data` - The WAV data to analyze
/// * `config` - Configuration parameters for the analysis
///
/// # Returns
/// * `Result<Vec<f32>, AudioError>` - Vector of normalized harmonic weights
///
/// # Errors
/// * If the configuration is invalid
/// * If there's an error during FFT processing
pub fn analyze_harmonics(
    wav_data: &WavData,
    config: &AnalysisConfig,
) -> Result<Vec<f32>, AudioError> {
    // Validate configuration
    config.validate(wav_data)?;

    // Extract mono samples for analysis
    let mono_samples = extract_mono_samples(wav_data, config)?;

    // Apply window function
    let windowed_samples = apply_hann_window(&mono_samples);

    // Perform FFT
    let spectrum = compute_fft(&windowed_samples)?;

    // Extract harmonics
    extract_harmonic_weights(&spectrum, config, wav_data.sample_rate)
}

/// Extracts mono samples from multi-channel WAV data.
///
/// If the input is multi-channel, the samples are averaged across all channels.
/// The extracted samples start at the configured start time and include the
/// specified number of samples.
///
/// # Arguments
/// * `wav_data` - The WAV data to extract samples from
/// * `config` - Configuration specifying start time and number of samples
///
/// # Returns
/// * `Result<Vec<f32>, AudioError>` - Vector of mono samples
///
/// # Errors
/// * If the requested sample range exceeds the file length
fn extract_mono_samples(
    wav_data: &WavData,
    config: &AnalysisConfig,
) -> Result<Vec<f32>, AudioError> {
    let start_sample = (config.start_time * wav_data.sample_rate as f32) as usize;
    let samples_per_channel = wav_data.samples.len() / wav_data.channels as usize;

    if start_sample + config.samples > samples_per_channel {
        return Err(AudioError::InvalidParams(
            "Sample range exceeds file length".to_string(),
        ));
    }

    let mut mono_samples = Vec::with_capacity(config.samples);

    // Average all channels if multi-channel
    for i in 0..config.samples {
        let sample_idx = (start_sample + i) * wav_data.channels as usize;
        let mut sum = 0.0;
        for ch in 0..wav_data.channels as usize {
            sum += wav_data.samples[sample_idx + ch];
        }
        mono_samples.push(sum / wav_data.channels as f32);
    }

    Ok(mono_samples)
}

/// Applies a Hann window function to the input samples.
///
/// The Hann window is used to reduce spectral leakage in the FFT analysis.
/// The window function is: w(n) = 0.5 * (1 - cos(2π*n/(N-1)))
///
/// # Arguments
/// * `samples` - Input samples to apply the window to
///
/// # Returns
/// * `Vec<f32>` - Windowed samples
fn apply_hann_window(samples: &[f32]) -> Vec<f32> {
    let len = samples.len();
    samples
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let window = 0.5 * (1.0 - (2.0 * PI * i as f32 / (len - 1) as f32).cos());
            sample * window
        })
        .collect()
}

/// Performs Fast Fourier Transform (FFT) on the input samples.
///
/// Converts the real-valued input samples to complex numbers and
/// performs an in-place FFT using the rustfft library.
///
/// # Arguments
/// * `samples` - Input samples to transform
///
/// # Returns
/// * `Result<Vec<Complex<f32>>, AudioError>` - Complex FFT spectrum
fn compute_fft(samples: &[f32]) -> Result<Vec<Complex<f32>>, AudioError> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(samples.len());

    // Convert samples to complex numbers
    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&x| Complex::new(x, 0.0)).collect();

    // Perform FFT
    fft.process(&mut buffer);

    Ok(buffer)
}

/// Extracts harmonic weights from the FFT spectrum.
///
/// For each harmonic:
/// 1. Calculates the target frequency bin
/// 2. Uses quadratic interpolation for precise magnitude
/// 3. Normalizes the magnitudes to [0, 1] range
/// 4. Applies the boost factor
/// 5. Rounds to 5 decimal places
///
/// # Arguments
/// * `spectrum` - FFT spectrum to analyze
/// * `config` - Analysis configuration
/// * `sample_rate` - Sample rate of the audio
///
/// # Returns
/// * `Result<Vec<f32>, AudioError>` - Vector of harmonic weights
///
/// # Errors
/// * If any harmonic frequency exceeds the Nyquist frequency
fn extract_harmonic_weights(
    spectrum: &[Complex<f32>],
    config: &AnalysisConfig,
    sample_rate: u32,
) -> Result<Vec<f32>, AudioError> {
    let freq_resolution = sample_rate as f32 / spectrum.len() as f32;
    let mut harmonics = Vec::with_capacity(config.num_harmonics);

    // Extract magnitude for each harmonic
    for k in 1..=config.num_harmonics {
        let target_freq = config.base_freq * k as f32;
        let bin = (target_freq / freq_resolution) as usize;

        if bin >= spectrum.len() - 1 {
            return Err(AudioError::InvalidParams(format!(
                "Harmonic {} exceeds Nyquist frequency",
                k
            )));
        }

        // Use quadratic interpolation for more precise magnitude
        let alpha = spectrum[bin - 1].norm();
        let beta = spectrum[bin].norm();
        let gamma = spectrum[bin + 1].norm();

        let p = if beta > 0.0 {
            0.5 * (alpha - gamma) / (alpha - 2.0 * beta + gamma)
        } else {
            0.0
        };

        let interpolated_magnitude = (beta - 0.25 * (alpha - gamma) * p).abs();
        harmonics.push(interpolated_magnitude);
    }

    // Normalize harmonics
    if let Some(&max_harmonic) = harmonics.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
        if max_harmonic > 0.0 {
            harmonics.iter_mut().for_each(|x| *x /= max_harmonic);
        }
    }

    // Apply boost factor
    harmonics.iter_mut().for_each(|x| *x *= config.boost);

    // Round to 5 decimal places
    harmonics
        .iter_mut()
        .for_each(|x| *x = (*x * 100000.0).round() / 100000.0);

    Ok(harmonics)
}
