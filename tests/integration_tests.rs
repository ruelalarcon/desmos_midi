// Integration tests for the Desmos MIDI converter
//
// These tests verify the end-to-end functionality of the MIDI to Desmos conversion pipeline.
// They focus on testing the complete process from loading a MIDI file to generating
// the final Desmos formula, ensuring all components work together correctly.
//
// The integration tests ensure:
// 1. Proper handling of MIDI files with soundfonts
// 2. Correct generation of Desmos formulas with expected format
// 3. The full conversion pipeline produces consistent and valid output

use desmos_midi::midi;
use std::path::Path;

// Import the test utils
mod test_utils;
use test_utils::{SAMPLES_DIR, SINE_SOUNDFONT, SQUARE_SOUNDFONT};

/// Test MIDI to Desmos formula conversion process.
///
/// This test verifies:
/// - Loading and processing of a MIDI file with soundfonts
/// - Conversion to the expected Desmos formula format
/// - Presence of all required formula components (A, B, C)
/// - Correct formatting of timestamps and note data
#[test]
fn test_midi_to_desmos_formula() {
    let midi_path = "tests/samples/c4_chromatic_60bpm.mid";

    // Process the MIDI file with soundfont to match CLI behavior using the custom samples directory
    let soundfonts = vec![String::from(SINE_SOUNDFONT)];
    let processor = midi::MidiProcessor::with_soundfont_dir(SAMPLES_DIR);
    let midi_result = processor
        .process_with_soundfonts(midi_path, soundfonts)
        .expect("Failed to process MIDI file");

    // Convert to Desmos formula string format
    let formula_string = midi_result.to_piecewise_function();

    // Check that it contains the expected formula components
    assert!(
        formula_string.contains("A=\\left\\{t<1.000:"),
        "Should contain A component with correct timestamps"
    );
    assert!(
        formula_string.contains("B=\\left[1\\right]"),
        "Should contain B component with correct harmonics"
    );
    assert!(
        formula_string.contains("C=1"),
        "Should contain C component with correct value"
    );
}

/// Test MIDI processing with soundfonts.
///
/// This test verifies:
/// - Loading and attaching soundfonts to a MIDI file
/// - Correct inclusion of soundfont data in the result
/// - Generation of valid Desmos formula with soundfont harmonics
#[test]
fn test_midi_with_soundfonts() {
    let midi_path = "tests/samples/c4_chromatic_60bpm.mid";
    let soundfonts = vec![String::from(SINE_SOUNDFONT)];

    // Process with soundfonts using the custom samples directory
    let processor = midi::MidiProcessor::with_soundfont_dir(SAMPLES_DIR);
    let result = processor
        .process_with_soundfonts(midi_path, soundfonts)
        .expect("Failed to process MIDI with soundfonts");

    // Check the soundfont data is included
    assert!(
        !result.soundfonts.fonts.is_empty(),
        "Should include soundfont data"
    );

    // Format and validate the output
    let formula_string = result.to_piecewise_function();
    assert!(
        formula_string.contains("B=\\left[1\\right]"),
        "Should contain harmonic data"
    );
}

/// Test CLI command line argument parsing.
///
/// This test verifies:
/// - Existence of CLI module for command processing
/// - (It's a lightweight test that just ensures the CLI structure exists)
#[test]
fn test_cli_parsing() {
    // This is a basic structural test to ensure CLI arguments are properly defined
    // We're not testing the actual CLI execution, just the structure

    // Check that the expected CLI commands exist in the right module
    let cli_module_path = Path::new("src/cli.rs");
    assert!(cli_module_path.exists(), "CLI module should exist");

    // We could add more specific tests here if needed
}

/// Test the entire pipeline from MIDI file to Desmos formula.
///
/// This comprehensive test verifies:
/// - Processing a complex MIDI file with multiple instruments
/// - Mapping different soundfonts to different MIDI channels
/// - Generating a complete Desmos formula with all expected components
/// - Format of complex note arrays with multiple instruments
/// - Inclusion of correct harmonic weights and counts
#[test]
fn test_end_to_end_pipeline() {
    // This test simulates the entire process from MIDI file to final Desmos formula

    // 1. Process MIDI file with soundfonts from the samples directory
    let midi_path = "tests/samples/c4c5_chromatic_piano_sax_dynamicbpm.mid";
    let soundfonts = vec![String::from(SINE_SOUNDFONT), String::from(SQUARE_SOUNDFONT)];

    let processor = midi::MidiProcessor::with_soundfont_dir(SAMPLES_DIR);
    let result = processor
        .process_with_soundfonts(midi_path, soundfonts)
        .expect("Failed to process MIDI file with soundfonts");

    // 2. Convert to Desmos formula and validate
    let formula = result.to_piecewise_function();

    // 3. Verify the expected output format
    // Expected formula components
    assert!(
        formula.contains("A=\\left\\{t<1.000:"),
        "Should contain notes array with correct start"
    );
    assert!(
        formula.contains("B=\\left[1,0,0,0,0,0,0,0,0,0,1,0,0.33333"),
        "Should contain harmonic weights"
    );
    assert!(formula.contains("C=10"), "Should contain harmonic count");

    // Verify output matches expected format for known input
    assert!(
        formula.contains("\\left[-9,100,0,3,100,1\\right]"),
        "Should contain first note data"
    );
    assert!(
        formula.contains("t<6.35:\\left[\\right]"),
        "Should contain proper ending"
    );
}
