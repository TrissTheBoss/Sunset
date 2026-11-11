use std::time::Duration;

pub fn frames_per_second(frames: u64, elapsed: Duration) -> f64 {
    let seconds = elapsed.as_secs_f64();
    if seconds > 0.0 {
        frames as f64 / seconds
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_frames_per_second() {
        assert_eq!(frames_per_second(1000, Duration::from_secs(1)), 1000.0);
        assert_eq!(frames_per_second(500, Duration::from_millis(250)), 2000.0);
        assert_eq!(frames_per_second(10, Duration::ZERO), 0.0);
    }
}
