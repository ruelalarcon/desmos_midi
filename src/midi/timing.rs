use super::types::TempoMap;

pub fn ticks_to_ms(ticks: u64, tempo_map: &TempoMap) -> u64 {
    let mut current_tick: u64 = 0;
    let mut current_time_us: u128 = 0; // Track time in microseconds for better precision
    let mut last_tempo_idx = 0;

    // Find the total time by accumulating through each tempo segment
    while current_tick < ticks {
        // Find the next tempo change or use the target ticks
        let next_tempo_change = tempo_map.changes.get(last_tempo_idx + 1)
            .map(|change| change.tick)
            .unwrap_or(ticks);

        let segment_end_tick = next_tempo_change.min(ticks);
        let tick_delta = segment_end_tick - current_tick;
        let current_tempo = tempo_map.changes[last_tempo_idx].tempo;

        // Calculate time in microseconds first
        current_time_us += (tick_delta as u128 * current_tempo as u128) /
            tempo_map.ticks_per_quarter as u128;

        if segment_end_tick == next_tempo_change {
            last_tempo_idx += 1;
        }
        current_tick = segment_end_tick;
    }

    // Convert microseconds to milliseconds at the end to maintain precision
    (current_time_us / 1000) as u64
}