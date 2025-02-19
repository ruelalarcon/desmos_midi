use super::types::TempoMap;

pub fn ticks_to_ms(ticks: u64, tempo_map: &TempoMap) -> u64 {
    let mut current_tick: u64 = 0;
    let mut current_time: u128 = 0;
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

        // Calculate time for this segment
        current_time += (tick_delta as u128 * current_tempo as u128) /
            (tempo_map.ticks_per_quarter as u128 * 1000);

        if segment_end_tick == next_tempo_change {
            last_tempo_idx += 1;
        }
        current_tick = segment_end_tick;
    }

    current_time as u64
}