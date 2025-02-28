// Basic MIDI types
/// Timestamp in milliseconds
pub type Timestamp = u64;
/// MIDI note number (0-127)
pub type MidiNote = u8;
/// Note velocity (0-127)
pub type Velocity = u8;
/// Number of semitones relative to A4 (440Hz)
pub type RelativeNote = i32;
/// Vector of harmonic weights for a particular instrument/sound
pub type SoundFont = Vec<f32>;

// Constants
/// Maximum length of a single Desmos formula section
/// Formulas longer than this will be split into multiple sections
const MAX_FORMULA_LENGTH: usize = 20000;

// Tempo handling
/// Represents a tempo change event in a MIDI file
#[derive(Debug, Clone)]
pub struct TempoChange {
    /// MIDI tick at which the tempo change occurs
    pub tick: u64,
    /// New tempo in microseconds per quarter note
    pub tempo: u32,
}

/// Map of tempo changes throughout a MIDI file
#[derive(Debug, Default)]
pub struct TempoMap {
    /// List of tempo changes in chronological order
    pub changes: Vec<TempoChange>,
    /// Number of MIDI ticks per quarter note
    pub ticks_per_quarter: u32,
}

impl TempoMap {
    /// Creates a new TempoMap with a default tempo of 120 BPM.
    ///
    /// # Arguments
    /// * `ticks_per_quarter` - Number of MIDI ticks per quarter note
    pub fn new(ticks_per_quarter: u32) -> Self {
        Self {
            changes: vec![TempoChange {
                tick: 0,
                tempo: 500000,
            }], // Default 120 BPM
            ticks_per_quarter,
        }
    }
}

// Channel and instrument information
/// Information about a MIDI channel
#[derive(Debug, Clone)]
pub struct Channel {
    /// Channel number (0-15)
    pub id: u8,
    /// MIDI program/instrument number (0-127)
    pub instrument: u8,
    /// Whether this is a drum channel (channel 10)
    pub is_drum: bool,
}

// Note events and timing
/// Represents a group of notes that start at the same time
#[derive(Debug)]
pub struct NoteEvent {
    /// Time in milliseconds when the notes start
    pub timestamp: Timestamp,
    /// List of (note, velocity, soundfont_index, end_time) tuples
    /// end_time is when this specific note should stop playing (in milliseconds)
    pub notes: Vec<(MidiNote, Velocity, usize, Timestamp)>,
}

// Soundfont handling
/// Collection of soundfonts with padding to ensure consistent length
#[derive(Debug)]
pub struct SoundFontMap {
    /// List of soundfonts, each padded to max_size
    pub fonts: Vec<SoundFont>,
    /// Length of the longest soundfont
    pub max_size: usize,
}

impl SoundFontMap {
    /// Creates a new SoundFontMap from a vector of soundfonts.
    /// All soundfonts are padded to match the length of the longest soundfont.
    ///
    /// # Arguments
    /// * `fonts` - Vector of soundfonts, each containing harmonic weights
    pub fn new(fonts: Vec<SoundFont>) -> Self {
        let max_size = fonts.iter().map(|f| f.len()).max().unwrap_or(0);
        // Pad all fonts to max_size
        let fonts: Vec<SoundFont> = fonts
            .into_iter()
            .map(|mut f| {
                f.resize(max_size, 0.0);
                f
            })
            .collect();
        Self { fonts, max_size }
    }
}

// Main song structure
/// Processed MIDI file ready for Desmos conversion
#[derive(Debug)]
pub struct ProcessedSong {
    /// List of note events in chronological order
    pub note_changes: Vec<NoteEvent>,
    /// List of channels used in the song
    pub channels: Vec<Channel>,
    /// Soundfonts assigned to each channel
    pub soundfonts: SoundFontMap,
}

impl ProcessedSong {
    /// Creates a Desmos-compatible piecewise function representation of the song.
    ///
    /// The output consists of three formulas:
    /// - `A`: Piecewise function containing note events with timestamps
    /// - `B`: Array containing all soundfont harmonic weights
    /// - `C`: Size of each soundfont (after padding)
    ///
    /// If the song is too long, it will be split into sections named A_1, A_2, etc.,
    /// with a main formula A that selects the appropriate section based on time.
    ///
    /// # Returns
    /// * `String` - Newline-separated Desmos formulas
    pub fn to_piecewise_function(&self) -> String {
        if self.note_changes.is_empty() {
            return "A=\\left\\{t<0:\\left[\\right]\\right\\}\nB=\\left[\\right]\nC=0".to_string();
        }

        let mut formulas = Vec::new();
        let mut current_section = Vec::new();
        let mut current_length = 0;
        let mut section_count = 0;
        let mut section_names = Vec::new();

        // Find all unique timestamps where notes start or end
        let timestamps = self.collect_all_timestamps();

        // For each timestamp, collect all active notes
        self.process_timestamps(
            &timestamps,
            &mut formulas,
            &mut current_section,
            &mut current_length,
            &mut section_count,
            &mut section_names,
        );

        // Add an empty array at the very end
        if let Some(&last_time) = timestamps.last() {
            current_section.push(format!("t<{}:\\left[\\right]", last_time + 0.1));
        }

        // Add the last section
        if !current_section.is_empty() {
            section_count += 1;
            let section_name = format!("A_{{{}}}", section_count);
            section_names.push(section_name.clone());

            let section_formula = format!(
                "{}=\\left\\{{{}\\right\\}}",
                section_name,
                current_section.join(",")
            );
            formulas.push(section_formula);
        }

        // Create the main formula
        if section_count > 1 {
            let main_formula = create_main_formula(&formulas, &section_names);
            formulas.insert(0, main_formula);
        } else {
            // If there's only one section, rename it to A
            formulas[0] = formulas[0].replace("A_{1}", "A");
        }

        // Add soundfont array (B) and max size (C)
        add_soundfont_formulas(&mut formulas, &self.soundfonts);

        formulas.join("\n")
    }

    /// Collects all timestamps where notes start or end.
    ///
    /// # Returns
    /// * `Vec<f64>` - Sorted vector of unique timestamps in seconds
    fn collect_all_timestamps(&self) -> Vec<f64> {
        let mut timestamps: Vec<f64> = self
            .note_changes
            .iter()
            .flat_map(|event| {
                let start = event.timestamp as f64 / 1000.0;
                let ends = event
                    .notes
                    .iter()
                    .map(|(_, _, _, end)| *end as f64 / 1000.0);
                std::iter::once(start).chain(ends)
            })
            .collect();
        timestamps.sort_by(|a, b| a.partial_cmp(b).unwrap());
        timestamps.dedup();
        timestamps
    }

    /// Processes timestamps and builds the piecewise formula sections.
    ///
    /// For each timestamp window, finds active notes and adds them to the formula.
    /// Splits the formula into sections if it exceeds the maximum length.
    fn process_timestamps(
        &self,
        timestamps: &[f64],
        formulas: &mut Vec<String>,
        current_section: &mut Vec<String>,
        current_length: &mut usize,
        section_count: &mut usize,
        section_names: &mut Vec<String>,
    ) {
        for window in timestamps.windows(2) {
            let current_time = window[0];
            let next_time = window[1];

            // Find all notes that are active at this time
            let active_notes = self.collect_active_notes(current_time);

            // Format the array of active notes
            let array_str = format_note_array_simple(&active_notes);
            let piece = format!("t<{:0.3}:{}", next_time, array_str);

            // Check if adding this piece would exceed the limit
            if *current_length + piece.len() > MAX_FORMULA_LENGTH && !current_section.is_empty() {
                *section_count += 1;
                let section_name = format!("A_{{{}}}", section_count);
                section_names.push(section_name.clone());

                let section_formula = format!(
                    "{}=\\left\\{{{}\\right\\}}",
                    section_name,
                    current_section.join(",")
                );
                formulas.push(section_formula);

                current_section.clear();
                *current_length = 0;
            }

            let piece_len = piece.len();
            current_section.push(piece);
            *current_length += piece_len;
        }
    }

    /// Collects all notes that are active at a given time.
    ///
    /// # Arguments
    /// * `current_time` - Time in seconds
    ///
    /// # Returns
    /// * `Vec<(MidiNote, Velocity, usize)>` - Vector of (note, velocity, soundfont_index) tuples
    fn collect_active_notes(&self, current_time: f64) -> Vec<(MidiNote, Velocity, usize)> {
        let mut active_notes = Vec::new();
        for event in &self.note_changes {
            let event_time = event.timestamp as f64 / 1000.0;
            if event_time <= current_time {
                // Add notes from this event that are still playing
                for &(note, vel, sf, end_time) in &event.notes {
                    if (end_time as f64 / 1000.0) > current_time {
                        active_notes.push((note, vel, sf));
                    }
                }
            }
        }

        // Sort notes for consistent output
        active_notes.sort_unstable_by_key(|(note, _, _)| *note);
        active_notes
    }
}

/// Formats a list of active notes into a Desmos array string.
///
/// # Arguments
/// * `notes` - List of (note, velocity, soundfont_index) tuples
///
/// # Returns
/// * `String` - Desmos array representation
fn format_note_array_simple(notes: &[(MidiNote, Velocity, usize)]) -> String {
    let note_array: Vec<String> = notes
        .iter()
        .flat_map(|&(note, velocity, soundfont_idx)| {
            vec![
                midi_note_to_relative(note).to_string(),
                velocity.to_string(),
                soundfont_idx.to_string(),
            ]
        })
        .collect();
    format!("\\left[{}\\right]", note_array.join(","))
}

/// Creates the main formula that selects the appropriate section based on time.
///
/// # Arguments
/// * `formulas` - Vector of section formulas
/// * `section_names` - Vector of section names (A_1, A_2, etc.)
///
/// # Returns
/// * `String` - Main formula that selects sections based on time
fn create_main_formula(formulas: &[String], section_names: &[String]) -> String {
    let mut main_pieces = Vec::new();

    for (i, section_formula) in formulas.iter().enumerate() {
        if let Some(last_time_str) = section_formula
            .split("t<")
            .filter(|s| s.contains(':'))
            .last()
            .and_then(|s| s.split(':').next())
        {
            if let Ok(end_time) = last_time_str.parse::<f64>() {
                main_pieces.push(format!("t<{:0.3}:{}", end_time, section_names[i]));
            }
        }
    }

    format!("A=\\left\\{{{}\\right\\}}", main_pieces.join(","))
}

/// Adds the soundfont array (B) and size (C) formulas.
///
/// # Arguments
/// * `formulas` - Vector to append formulas to
/// * `soundfonts` - SoundFontMap containing the soundfonts
fn add_soundfont_formulas(formulas: &mut Vec<String>, soundfonts: &SoundFontMap) {
    let soundfont_values: Vec<String> = soundfonts
        .fonts
        .iter()
        .flat_map(|font| font.iter().map(|v| v.to_string()))
        .collect();
    formulas.push(format!("B=\\left[{}\\right]", soundfont_values.join(",")));
    formulas.push(format!("C={}", soundfonts.max_size));
}

/// Converts a MIDI note number to a relative note value.
///
/// The relative value is the number of semitones from A4 (440Hz).
/// For example:
/// - A4 (MIDI note 69) -> 0
/// - A#4 (MIDI note 70) -> 1
/// - G#4 (MIDI note 68) -> -1
///
/// # Arguments
/// * `note` - MIDI note number (0-127)
///
/// # Returns
/// * `RelativeNote` - Number of semitones from A4
fn midi_note_to_relative(note: MidiNote) -> RelativeNote {
    (note as RelativeNote) - 69 // A (MIDI note 69 / 440 Hz) as root note (0)
}

/// Custom error type for MIDI processing
#[derive(Debug, thiserror::Error)]
pub enum MidiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("MIDI parsing error: {0}")]
    MidiParse(#[from] midly::Error),

    #[error("Invalid soundfont: {0}")]
    InvalidSoundfont(String),

    #[error("Unsupported MIDI timing format")]
    UnsupportedTimingFormat,

    #[error("Soundfont mismatch: {0}")]
    SoundfontMismatch(String),

    #[error("Parsing error: {0}")]
    Parse(#[from] std::num::ParseFloatError),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    #[error("Other error: {0}")]
    #[allow(dead_code)]
    Other(String),
}
