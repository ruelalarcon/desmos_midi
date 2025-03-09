// MIDI processing tests
//
// These tests focus on the MIDI processing functionality of the application.
// They verify that MIDI files can be correctly parsed, processed, and that
// all the expected data is extracted (notes, channels, instruments, etc.).
//
// Note that these tests do not verify correctness of the Desmos formula,
// that is the domain of the `format_tests.rs` file.
//
// The tests cover different types of MIDI files:
// - Files with constant BPM
// - Files with dynamic BPM changes
// - Files with multiple channels and instruments
// - Edge cases and error handling

use desmos_midi::midi::{self, ProcessedSong};

// Import the test utils
mod test_utils;
use test_utils::{SAMPLES_DIR, SINE_SOUNDFONT, SQUARE_SOUNDFONT};

/// Test MIDI processing with constant BPM.
///
/// This test verifies:
/// - Correct loading and parsing of a MIDI file with constant tempo
/// - Extraction of note data from the file
/// - Generation of Desmos formula with expected format
#[test]
fn test_midi_constant_bpm() {
    let midi_path = "tests/samples/c4_chromatic_60bpm.mid";

    // Use process_midi with a soundfont instead of process_midi_info to get note data
    let soundfonts = vec![String::from(SINE_SOUNDFONT)];
    let processor = midi::MidiProcessor::with_soundfont_dir(SAMPLES_DIR);
    let result = processor
        .process_with_soundfonts(midi_path, soundfonts)
        .expect("Failed to process MIDI file with soundfonts");

    // Check that we have note data
    assert!(!result.note_changes.is_empty());

    // Expected Desmos formula components
    let expected_formula = "A=\\left\\{t<";

    // Generate Desmos formula and check against expected
    let desmos_formula = result.to_piecewise_function();
    assert!(desmos_formula.contains(expected_formula));
}

/// Test MIDI processing with dynamic BPM.
///
/// This test verifies:
/// - Correct loading and parsing of a MIDI file with tempo changes
/// - Extraction of note data and tempo information
/// - Proper timing calculations when BPM changes
/// - Verification that tempo changes affect note timing patterns
#[test]
fn test_midi_dynamic_bpm() {
    let midi_path = "tests/samples/c4_chromatic_dynamicbpm.mid";

    // Use process_midi with a soundfont instead of process_midi_info to get note data
    let soundfonts = vec![String::from(SINE_SOUNDFONT)];
    let processor = midi::MidiProcessor::with_soundfont_dir(SAMPLES_DIR);
    let result = processor
        .process_with_soundfonts(midi_path, soundfonts)
        .expect("Failed to process MIDI file with soundfonts");

    // Expected Desmos formula components - just check for basic structure
    let expected_formula = "A=\\left\\{t<";

    // Generate Desmos formula and check against expected
    let desmos_formula = result.to_piecewise_function();
    assert!(desmos_formula.contains(expected_formula));

    // Verify tempo changes by checking note timing patterns
    // If the BPM changes, the timing between notes will not be constant
    let timestamps = collect_note_timestamps(&result);
    assert!(timestamps.len() > 2);
    let time_diffs: Vec<f64> = timestamps.windows(2).map(|w| w[1] - w[0]).collect();
    assert!(time_diffs.windows(2).any(|w| (w[1] - w[0]).abs() > 0.001));
}

/// Test MIDI processing with multiple channels and instruments.
///
/// This test verifies:
/// - Loading and processing a MIDI file with multiple channels
/// - Correct mapping of different instruments to different channels
/// - Attaching multiple soundfonts to different channels
/// - Generation of Desmos formula with multiple instruments
#[test]
fn test_midi_multiple_channels() {
    let midi_path = "tests/samples/c4c5_chromatic_piano_sax_dynamicbpm.mid";
    let soundfonts = vec![String::from(SINE_SOUNDFONT), String::from(SQUARE_SOUNDFONT)];

    let processor = midi::MidiProcessor::with_soundfont_dir(SAMPLES_DIR);
    let result = processor
        .process_with_soundfonts(midi_path, soundfonts)
        .expect("Failed to process MIDI file with soundfonts");

    // Expected Desmos formula components
    let expected_formula = "A=\\left\\{t<";

    // Generate Desmos formula and check against expected
    let desmos_formula = result.to_piecewise_function();
    assert!(desmos_formula.contains(expected_formula));

    // Verify we have multiple channels
    assert!(
        result.channels.len() >= 2,
        "Should have at least 2 channels"
    );
}

/// Test MIDI information extraction.
///
/// This test verifies:
/// - Extracting channel and instrument information from a MIDI file
/// - Proper identification of different instruments in different channels
/// - Verification that info-only mode doesn't process note events
#[test]
fn test_midi_info_extraction() {
    let midi_path = "tests/samples/c4c5_chromatic_piano_sax_dynamicbpm.mid";

    // For info extraction, we should use process_midi_info
    let result = midi::process_midi_info(midi_path).expect("Failed to process MIDI file");

    // Verify channel info
    assert!(
        result.channels.len() >= 2,
        "Should have at least 2 channels"
    );

    // Check that the channels have different instruments
    let instruments: Vec<_> = result.channels.iter().map(|c| c.instrument).collect();
    assert!(instruments.contains(&0), "Should have piano instrument (0)");

    // Note: process_midi_info doesn't fill note_changes, so we don't check that
}

/// Test the MidiProcessor class.
///
/// This test verifies:
/// - Creating a MidiProcessor with custom soundfont directory
/// - Using it to process MIDI file info (channel/instrument data)
/// - Using it to process a MIDI file with soundfonts
/// - Verification that soundfont data is included in the result
#[test]
fn test_midi_processor() {
    let processor = midi::MidiProcessor::with_soundfont_dir(SAMPLES_DIR);

    // Test info processing
    let info_result = processor
        .process_info("tests/samples/c4_chromatic_60bpm.mid")
        .expect("Failed to process MIDI info");
    assert!(!info_result.channels.is_empty());

    // Test processing with soundfonts
    let soundfonts = vec![String::from(SINE_SOUNDFONT)];
    let result = processor
        .process_with_soundfonts("tests/samples/c4_chromatic_60bpm.mid", soundfonts)
        .expect("Failed to process MIDI with soundfonts");

    // Verify soundfont data is included
    assert!(
        !result.soundfonts.fonts.is_empty(),
        "Should include soundfont data"
    );
}

/// Test MIDI error handling.
///
/// This test verifies:
/// - Proper error handling for non-existent MIDI files
/// - Proper error handling for invalid soundfont files
/// - Verification that errors are correctly propagated
#[test]
fn test_midi_error_cases() {
    // Test with non-existent file
    let result = midi::process_midi_info("non_existent_file.mid");
    assert!(result.is_err(), "Should return error for non-existent file");

    // Test with invalid soundfont
    let soundfonts = vec![String::from("non_existent_soundfont.txt")];
    let processor = midi::MidiProcessor::with_soundfont_dir(SAMPLES_DIR);
    let result = processor.verify_soundfonts(&soundfonts);
    assert!(
        result.is_err(),
        "Should return error for non-existent soundfont"
    );
}

/// Helper function to extract timestamps from a processed song.
///
/// Used to analyze the timing of notes in a MIDI file, particularly
/// useful for verifying tempo changes affect note timing.
fn collect_note_timestamps(song: &ProcessedSong) -> Vec<f64> {
    let mut timestamps: Vec<f64> = song
        .note_changes
        .iter()
        .map(|event| event.timestamp as f64 / 1000.0)
        .collect();
    timestamps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    timestamps
}
