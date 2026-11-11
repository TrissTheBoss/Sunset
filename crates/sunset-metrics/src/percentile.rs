pub fn percentile(values: &[f32], p: f64) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let rank = (p * sorted.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted.len() - 1);
    Some(sorted[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_known_values() {
        let values = [10.0, 3.0, 1.0, 8.0, 5.0, 7.0, 2.0, 9.0, 4.0, 6.0];
        assert_eq!(percentile(&values, 0.0), Some(1.0));
        assert_eq!(percentile(&values, 0.5), Some(5.0));
        assert_eq!(percentile(&values, 0.9), Some(9.0));
        assert_eq!(percentile(&values, 1.0), Some(10.0));
    }

    #[test]
    fn percentile_matches_naive_reference() {
        let mut state = 0x9e3779b97f4a7c15u64;
        for _ in 0..500 {
            let n = (next_u64(&mut state) % 64) as usize;
            let mut values = Vec::with_capacity(n);
            for _ in 0..n {
                values.push((next_u64(&mut state) % 1000) as f32);
            }
            for p in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99, 1.0] {
                assert_eq!(percentile(&values, p), naive_percentile(&values, p));
            }
        }
    }

    fn naive_percentile(values: &[f32], p: f64) -> Option<f32> {
        if values.is_empty() {
            return None;
        }
        let mut sorted: Vec<f32> = values.to_vec();
        sorted.sort_by(f32::total_cmp);
        let rank = (p * sorted.len() as f64).ceil().max(1.0) as usize;
        Some(sorted[rank - 1])
    }

    fn next_u64(state: &mut u64) -> u64 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        x
    }
}
