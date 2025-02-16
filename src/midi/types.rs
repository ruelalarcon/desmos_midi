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
    pub fn to_timing_data(&self) -> Vec<Timestamp> {
        self.note_changes
            .iter()
            .map(|event| event.timestamp)
            .collect()
    }

    pub fn to_formulas(&self) -> Vec<String> {
        self.note_changes
            .iter()
            .map(|event| {
                let relative_notes: Vec<RelativeNote> = event.notes
                    .iter()
                    .map(|&n| midi_note_to_relative(n))
                    .collect();
                format!("A\\to\\left[{}\\right]",
                       relative_notes.iter()
                           .map(|n| n.to_string())
                           .collect::<Vec<String>>()
                           .join(","))
            })
            .collect()
    }
}

fn midi_note_to_relative(note: MidiNote) -> RelativeNote {
    (note as RelativeNote) - 58 // Bb as root note (0)
}