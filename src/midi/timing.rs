pub fn ticks_to_ms(ticks: u64, tempo: u32, ticks_per_quarter: u32) -> u64 {
    // Convert MIDI ticks to milliseconds
    // tempo is in microseconds per quarter note
    // Formula: (ticks * tempo) / (ticks_per_quarter * 1000)
    (ticks as u128 * tempo as u128 / (ticks_per_quarter as u128 * 1000)) as u64
}