pub type Timestamp = u64;
pub type MidiNote = u8;
pub type RelativeNote = i32;

#[derive(Debug)]
pub struct NoteEvent {
    pub timestamp: Timestamp,
    pub notes: Vec<MidiNote>,
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

        let mut pieces = Vec::new();

        // Add initial empty array at t=0 if the first note doesn't start at 0
        if self.note_changes[0].timestamp > 0 {
            pieces.push("t<0:\\left[\\right]".to_string());
        }

        for i in 0..self.note_changes.len() {
            let event = &self.note_changes[i];
            let relative_notes: Vec<RelativeNote> = event.notes
                .iter()
                .map(|&n| midi_note_to_relative(n))
                .collect();

            let array_str = format!("\\left[{}\\right]",
                relative_notes.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(","));

            // Get the end time for this note array (start time of next array)
            let end_time = if i < self.note_changes.len() - 1 {
                self.note_changes[i + 1].timestamp as f64 / 1000.0
            } else {
                // For the last note, use its own timestamp plus a small duration
                // This ensures the last note plays for at least some time
                (event.timestamp as f64 / 1000.0) + 0.001
            };

            pieces.push(format!("t<{}:{}", end_time, array_str));
        }

        format!("A=\\left\\{{{}\\right\\}}", pieces.join(","))
    }
}

fn midi_note_to_relative(note: MidiNote) -> RelativeNote {
    (note as RelativeNote) - 58 // Bb (MIDI note 58) as root note (0)
}