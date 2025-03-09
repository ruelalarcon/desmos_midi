// Test utilities and common constants
//
// This file provides shared utilities and constants used across multiple test files.
// It centralizes file paths, sample locations, and helper functions to avoid
// duplication and ensure consistency across tests.
//
// The utilities include:
// - Constants for test file paths
// - Constants for soundfont paths
// - Helper functions for file validation
// - Test environment validation

use std::path::Path;

/// Constants for test file paths
pub const MIDI_C4_CHROMATIC_60BPM: &str = "tests/samples/c4_chromatic_60bpm.mid";
pub const MIDI_C4_CHROMATIC_DYNAMICBPM: &str = "tests/samples/c4_chromatic_dynamicbpm.mid";
pub const MIDI_C4C5_CHROMATIC_PIANO_SAX_DYNAMICBPM: &str =
    "tests/samples/c4c5_chromatic_piano_sax_dynamicbpm.mid";
pub const WAV_440HZ_5SEC: &str = "tests/samples/440hz_44100hz_16bit_5sec.wav";

/// Constants for soundfont locations
pub const SAMPLES_DIR: &str = "tests/samples";
pub const SINE_SOUNDFONT: &str = "sine.txt";
pub const SQUARE_SOUNDFONT: &str = "square.txt";
pub const SINE_SOUNDFONT_PATH: &str = "tests/samples/sine.txt";
pub const SQUARE_SOUNDFONT_PATH: &str = "tests/samples/square.txt";

/// Check if a file exists at the specified path.
///
/// # Arguments
/// * `path` - The file path to check
///
/// # Returns
/// * `bool` - True if the file exists, false otherwise
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Parse a comma-separated string of harmonic values into a Vec<f32>.
///
/// Used for converting the harmonic string representation into a vector of floats
/// for further processing and analysis.
///
/// # Arguments
/// * `harmonics_str` - Comma-separated string of harmonic values
///
/// # Returns
/// * `Vec<f32>` - Vector of parsed float values
#[allow(dead_code)]
pub fn parse_harmonics(harmonics_str: &str) -> Vec<f32> {
    harmonics_str
        .split(',')
        .map(|s| s.parse::<f32>().unwrap_or(0.0))
        .collect()
}

/// Check if a string contains all components of a valid Desmos formula.
///
/// A valid Desmos formula should contain:
/// - An A component with piecewise function for notes
/// - A B component with harmonic weights array
/// - A C component with harmonic count
///
/// # Arguments
/// * `formula` - The formula string to check
///
/// # Returns
/// * `bool` - True if the formula contains all required components
#[allow(dead_code)]
pub fn check_desmos_formula_format(formula: &str) -> bool {
    formula.contains("A=\\left\\{") && formula.contains("B=\\left[") && formula.contains("C=")
}

/// Validate that all test files exist.
///
/// This function checks that all the necessary test files are present
/// in the tests/samples directory before running tests that depend on them.
///
/// # Panics
/// * If any required test file is missing
pub fn validate_test_files() {
    // Check that all test files exist
    assert!(
        file_exists(MIDI_C4_CHROMATIC_60BPM),
        "Missing test file: {}",
        MIDI_C4_CHROMATIC_60BPM
    );
    assert!(
        file_exists(MIDI_C4_CHROMATIC_DYNAMICBPM),
        "Missing test file: {}",
        MIDI_C4_CHROMATIC_DYNAMICBPM
    );
    assert!(
        file_exists(MIDI_C4C5_CHROMATIC_PIANO_SAX_DYNAMICBPM),
        "Missing test file: {}",
        MIDI_C4C5_CHROMATIC_PIANO_SAX_DYNAMICBPM
    );
    assert!(
        file_exists(WAV_440HZ_5SEC),
        "Missing test file: {}",
        WAV_440HZ_5SEC
    );

    // Check soundfont files
    assert!(
        file_exists(SINE_SOUNDFONT_PATH),
        "Missing soundfont file: {}",
        SINE_SOUNDFONT_PATH
    );
    assert!(
        file_exists(SQUARE_SOUNDFONT_PATH),
        "Missing soundfont file: {}",
        SQUARE_SOUNDFONT_PATH
    );
}

/// Test that verifies the test environment is correctly set up.
///
/// This meta-test ensures that all required test files exist
/// before other tests are run.
#[test]
fn test_validate_test_environment() {
    // This is a meta-test to ensure our test environment is correctly set up
    validate_test_files();
}
