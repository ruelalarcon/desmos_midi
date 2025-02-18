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

        for i in 0..self.note_changes.len() {
            let event = &self.note_changes[i];
            let time = event.timestamp as f64 / 1000.0;

            let relative_notes: Vec<RelativeNote> = event.notes
                .iter()
                .map(|&n| midi_note_to_relative(n))
                .collect();

            let array_str = format!("\\left[{}\\right]",
                relative_notes.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(","));

            pieces.push(format!("t<{}:{}", time, array_str));
        }

        format!("A=\\left\\{{{}\\right\\}}", pieces.join(","))
    }
}

fn midi_note_to_relative(note: MidiNote) -> RelativeNote {
    (note as RelativeNote) - 60 // Middle C (60) as root note (0)
}