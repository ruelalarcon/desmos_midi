use super::types::{AnalysisConfig, AudioError, WavData};
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;

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

fn compute_fft(samples: &[f32]) -> Result<Vec<Complex<f32>>, AudioError> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(samples.len());

    // Convert samples to complex numbers
    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&x| Complex::new(x, 0.0)).collect();

    // Perform FFT
    fft.process(&mut buffer);

    Ok(buffer)
}

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

        let interpolated_magnitude = beta - 0.25 * (alpha - gamma) * p;
        harmonics.push(interpolated_magnitude);
    }

    // Normalize harmonics
    if let Some(&max_harmonic) = harmonics.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
        if max_harmonic > 0.0 {
            harmonics.iter_mut().for_each(|x| *x /= max_harmonic);
        }
    }

    Ok(harmonics)
}
