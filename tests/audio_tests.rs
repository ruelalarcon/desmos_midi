// Audio processing tests
//
// These tests focus on the audio processing functionality of the application.
// They verify the loading and analysis of WAV files to extract harmonic content,
// which is used to create soundfonts for the MIDI to Desmos conversion.
//
// The tests cover:
// - WAV file loading and validation
// - Harmonic analysis of audio data
// - Configuration validation for analysis parameters
// - Error handling for invalid parameters and files

use desmos_midi::audio;
use std::path::Path;

/// Test WAV file loading functionality.
///
/// This test verifies:
/// - Loading a WAV file from disk
/// - Correct extraction of WAV properties (sample rate, channels)
/// - Successful conversion to normalized sample format
#[test]
fn test_wav_file_loading() {
    let wav_path = Path::new("tests/samples/440hz_44100hz_16bit_5sec.wav");
    let result = audio::read_wav_file(wav_path).expect("Failed to load WAV file");

    // Check basic WAV properties
    assert_eq!(result.sample_rate, 44100);
    assert!(result.channels > 0);
    assert!(!result.samples.is_empty());
}

/// Test harmonic analysis of a sine wave.
///
/// This test verifies:
/// - Analysis of harmonic content in a 440Hz sine wave
/// - Extraction of harmonic weights with FFT analysis
/// - Verification that sine wave has mainly fundamental harmonic
#[test]
fn test_harmonic_analysis() {
    let wav_path = Path::new("tests/samples/440hz_44100hz_16bit_5sec.wav");
    let wav_data = audio::read_wav_file(wav_path).expect("Failed to load WAV file");

    let config = audio::AnalysisConfig {
        samples: 8192,
        start_time: 0.0,
        base_freq: 440.0,
        num_harmonics: 16,
        boost: 1.0,
    };

    let harmonics =
        audio::analyze_harmonics(&wav_data, &config).expect("Failed to analyze harmonics");

    // The expected harmonics for a 440Hz sine wave
    let expected_harmonics = vec![
        1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ];

    // Check that we have the right number of harmonics
    assert_eq!(harmonics.len(), expected_harmonics.len());

    // Check that the first harmonic (fundamental) is dominant
    assert!(harmonics[0] > 0.9, "First harmonic should be close to 1.0");

    // Check that higher harmonics are very low (sine wave has minimal harmonics)
    for i in 1..harmonics.len() {
        assert!(
            harmonics[i] < 0.1,
            "Higher harmonics should be close to 0.0"
        );
    }
}

/// Test analysis configuration validation.
///
/// This test verifies:
/// - Validation of analysis parameters against WAV file constraints
/// - Detection of invalid start time (beyond file duration)
/// - Error reporting for invalid parameters
#[test]
fn test_analysis_config_validation() {
    let wav_path = Path::new("tests/samples/440hz_44100hz_16bit_5sec.wav");
    let wav_data = audio::read_wav_file(wav_path).expect("Failed to load WAV file");

    // Valid config
    let valid_config = audio::AnalysisConfig {
        samples: 8192,
        start_time: 0.0,
        base_freq: 440.0,
        num_harmonics: 16,
        boost: 1.0,
    };
    assert!(valid_config.validate(&wav_data).is_ok());

    // Invalid config: start_time too large
    let invalid_config = audio::AnalysisConfig {
        samples: 8192,
        start_time: 10.0, // Way beyond the 5-second sample
        base_freq: 440.0,
        num_harmonics: 16,
        boost: 1.0,
    };
    assert!(invalid_config.validate(&wav_data).is_err());
}

/// Test audio processing error cases.
///
/// This test verifies:
/// - Proper error handling for non-existent WAV files
/// - Error handling when harmonics exceed Nyquist frequency
/// - Error propagation from analysis functions
#[test]
fn test_audio_error_cases() {
    // Test with non-existent file
    let wav_path = Path::new("non_existent_file.wav");
    let result = audio::read_wav_file(wav_path);
    assert!(result.is_err(), "Should return error for non-existent file");

    // Load a valid WAV file for further error testing
    let wav_path = Path::new("tests/samples/440hz_44100hz_16bit_5sec.wav");
    let wav_data = audio::read_wav_file(wav_path).expect("Failed to load WAV file");

    // Test with high frequency harmonic beyond the Nyquist frequency
    let invalid_nyquist_config = audio::AnalysisConfig {
        samples: 8192,
        start_time: 0.0,
        base_freq: 5000.0, // High base frequency
        num_harmonics: 16, // With 16 harmonics will exceed Nyquist frequency
        boost: 1.0,
    };

    // Analyze harmonics should fail with high frequency harmonics
    let result = audio::analyze_harmonics(&wav_data, &invalid_nyquist_config);
    assert!(
        result.is_err(),
        "Should return error for harmonics exceeding Nyquist frequency"
    );
}
