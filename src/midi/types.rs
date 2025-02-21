// Basic MIDI types
pub type Timestamp = u64;
pub type MidiNote = u8;
pub type Velocity = u8;
pub type RelativeNote = i32;
pub type SoundFont = Vec<f32>;

// Constants
const MAX_FORMULA_LENGTH: usize = 20000;

// Tempo handling
#[derive(Debug, Clone)]
pub struct TempoChange {
    pub tick: u64,
    pub tempo: u32,
}

#[derive(Debug, Default)]
pub struct TempoMap {
    pub changes: Vec<TempoChange>,
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
#[derive(Debug, Clone)]
pub struct Channel {
    pub id: u8,
    pub instrument: u8,
    pub is_drum: bool,
}

// Note events and timing
#[derive(Debug)]
pub struct NoteEvent {
    pub timestamp: Timestamp,
    /// Tuple of (note, velocity, soundfont_index, end_time)
    /// end_time is when this specific note should stop playing (in milliseconds)
    pub notes: Vec<(MidiNote, Velocity, usize, Timestamp)>,
}

// Soundfont handling
#[derive(Debug)]
pub struct SoundFontMap {
    pub fonts: Vec<SoundFont>,
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
#[derive(Debug)]
pub struct ProcessedSong {
    pub note_changes: Vec<NoteEvent>,
    pub channels: Vec<Channel>,
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

        // For each timestamp, collect all active notes
        for window in timestamps.windows(2) {
            let current_time = window[0];
            let next_time = window[1];

            // Find all notes that are active at this time
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

            // Format the array of active notes
            let array_str = format_note_array_simple(&active_notes);
            let piece = format!("t<{:0.3}:{}", next_time, array_str);

            // Check if adding this piece would exceed the limit
            if current_length + piece.len() > MAX_FORMULA_LENGTH && !current_section.is_empty() {
                section_count += 1;
                let section_name = format!("A_{{{}}}", section_count);
                section_names.push(section_name.clone());

                let section_formula = format!(
                    "{}=\\left\\{{{}\\right\\}}",
                    section_name,
                    current_section.join(",")
                );
                formulas.push(section_formula);

                current_section.clear();
                current_length = 0;
            }

            let piece_len = piece.len();
            current_section.push(piece);
            current_length += piece_len;
        }

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
