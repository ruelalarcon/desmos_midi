pub type Timestamp = u64;
pub type MidiNote = u8;
pub type Velocity = u8;
pub type RelativeNote = i32;

const MAX_FORMULA_LENGTH: usize = 20000;

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
    pub fn new(ticks_per_quarter: u32) -> Self {
        Self {
            changes: vec![TempoChange { tick: 0, tempo: 500000 }], // Default 120 BPM
            ticks_per_quarter,
        }
    }
}

#[derive(Debug)]
pub struct NoteEvent {
    pub timestamp: Timestamp,
    pub notes: Vec<(MidiNote, Velocity)>,
}

#[derive(Debug)]
pub struct ProcessedSong {
    pub note_changes: Vec<NoteEvent>,
}

impl ProcessedSong {
    pub fn to_piecewise_function(&self) -> String {
        if self.note_changes.is_empty() {
            return "A=\\left\\{t<0:\\left[\\right]\\right\\}".to_string();
        }

        let mut formulas = Vec::new();
        let mut current_section = Vec::new();
        let mut current_length = 0;
        let mut section_count = 0;
        let mut section_names = Vec::new();

        // Helper function to format a note array
        let format_note_array = |event: &NoteEvent| -> String {
            let note_array: Vec<String> = event.notes
                .iter()
                .flat_map(|&(note, velocity)| {
                    vec![
                        midi_note_to_relative(note).to_string(),
                        velocity.to_string()
                    ]
                })
                .collect();
            format!("\\left[{}\\right]", note_array.join(","))
        };

        // Process events and split into sections
        for i in 0..self.note_changes.len() {
            let event = &self.note_changes[i];
            let end_time = if i < self.note_changes.len() - 1 {
                self.note_changes[i + 1].timestamp as f64 / 1000.0
            } else {
                (event.timestamp as f64 / 1000.0) + 0.001
            };

            let array_str = format_note_array(event);
            let piece = format!("t<{}:{}", end_time, array_str);

            // Check if adding this piece would exceed the limit
            if current_length + piece.len() > MAX_FORMULA_LENGTH && !current_section.is_empty() {
                // Create a new section
                section_count += 1;
                let section_name = format!("A_{{{}}}", section_count);
                section_names.push(section_name.clone());

                let section_formula = format!("{}=\\left\\{{{}\\right\\}}",
                    section_name,
                    current_section.join(","));
                formulas.push(section_formula);

                current_section.clear();
                current_length = 0;
            }

            current_section.push(piece.clone());
            current_length += piece.len();
        }

        // Add the last section if it's not empty
        if !current_section.is_empty() {
            section_count += 1;
            let section_name = format!("A_{{{}}}", section_count);
            section_names.push(section_name.clone());

            let section_formula = format!("{}=\\left\\{{{}\\right\\}}",
                section_name,
                current_section.join(","));
            formulas.push(section_formula);
        }

        // Create the main formula that references all sections
        if section_count > 1 {
            let mut main_pieces = Vec::new();

            // Find the last timestamp in each section's formula
            for (i, section_formula) in formulas.iter().enumerate() {
                // Extract the last timestamp from the section formula
                if let Some(last_time_str) = section_formula
                    .split("t<")
                    .filter(|s| s.contains(':'))
                    .last()
                    .and_then(|s| s.split(':').next())
                {
                    if let Ok(end_time) = last_time_str.parse::<f64>() {
                        main_pieces.push(format!("t<{}:{}", end_time, section_names[i]));
                    }
                }
            }

            let main_formula = format!("A=\\left\\{{{}\\right\\}}", main_pieces.join(","));
            formulas.insert(0, main_formula);
        } else {
            // If there's only one section, rename it to A
            formulas[0] = formulas[0].replace("A_{1}", "A");
        }

        formulas.join("\n")
    }
}

fn midi_note_to_relative(note: MidiNote) -> RelativeNote {
    (note as RelativeNote) - 58 // Bb (MIDI note 58) as root note (0)
}