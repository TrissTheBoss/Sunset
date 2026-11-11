use std::process::ExitCode;
use std::time::Duration;

pub fn run_cli(suite: &str) -> ExitCode {
    match suite {
        "harness" => run_harness(),
        other => {
            eprintln!("test: unknown suite '{other}' (available: harness)");
            ExitCode::from(2)
        }
    }
}

fn run_harness() -> ExitCode {
    let probes = [
        probe_allocation_in_frame_scope(),
        probe_two_millisecond_stall(),
        probe_timed_out_child(),
        probe_changed_hash(),
    ];
    let mut all_detected = true;
    for probe in &probes {
        match probe {
            ProbeResult::Detected { name, reason } => {
                println!("harness: DETECTED {name} ({reason})");
            }
            ProbeResult::Missed { name, reason } => {
                all_detected = false;
                eprintln!("harness: MISSED {name} ({reason})");
            }
        }
    }
    if all_detected {
        println!("harness: {} fault(s) detected", probes.len());
        ExitCode::from(0)
    } else {
        ExitCode::from(1)
    }
}

enum ProbeResult {
    Detected { name: &'static str, reason: String },
    Missed { name: &'static str, reason: String },
}

fn probe_allocation_in_frame_scope() -> ProbeResult {
    let (counts, _) = sunset_metrics::count_alloc(|| {
        let value = Box::new(0u8);
        std::hint::black_box(&value);
    });
    if counts.0 > 0 {
        ProbeResult::Detected {
            name: "allocation-in-frame-scope",
            reason: format!("counting allocator recorded {} allocation(s)", counts.0),
        }
    } else {
        ProbeResult::Missed {
            name: "allocation-in-frame-scope",
            reason: "counting allocator recorded 0 allocations for an allocating scope".to_string(),
        }
    }
}

fn probe_two_millisecond_stall() -> ProbeResult {
    let mut ring = sunset_metrics::IntervalRing::new(64);
    for _ in 0..32 {
        ring.push(0.3);
    }
    ring.push(2.0);
    let max_interval = ring.values().into_iter().fold(0.0f32, f32::max);
    if max_interval > 0.50 {
        ProbeResult::Detected {
            name: "2ms-stall",
            reason: format!("max frame interval {max_interval:.3} ms exceeds the 0.50 ms budget"),
        }
    } else {
        ProbeResult::Missed {
            name: "2ms-stall",
            reason: format!("max frame interval {max_interval:.3} ms stayed within the budget"),
        }
    }
}

fn probe_timed_out_child() -> ProbeResult {
    let outcome =
        crate::runner::run_child("sleep", &["30".to_string()], Duration::from_millis(100));
    match outcome {
        crate::runner::RunOutcome::TimedOut => ProbeResult::Detected {
            name: "timed-out-child",
            reason: "runner returned TimedOut for a hanging child".to_string(),
        },
        other => ProbeResult::Missed {
            name: "timed-out-child",
            reason: format!("runner returned {other:?} instead of TimedOut"),
        },
    }
}

fn probe_changed_hash() -> ProbeResult {
    let original = crate::lock::hex_sha256(b"sunset reference content");
    let changed = crate::lock::hex_sha256(b"sunset reference content!");
    if original != changed {
        ProbeResult::Detected {
            name: "changed-hash",
            reason: format!(
                "SHA-256 differs ({}... vs {}...)",
                &original[..16],
                &changed[..16]
            ),
        }
    } else {
        ProbeResult::Missed {
            name: "changed-hash",
            reason: "SHA-256 produced identical hashes for differing content".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_detected(result: &ProbeResult) -> bool {
        matches!(result, ProbeResult::Detected { .. })
    }

    #[test]
    fn allocation_fault_is_detected() {
        assert!(is_detected(&probe_allocation_in_frame_scope()));
    }

    #[test]
    fn stall_fault_is_detected() {
        assert!(is_detected(&probe_two_millisecond_stall()));
    }

    #[test]
    fn timeout_fault_is_detected() {
        assert!(is_detected(&probe_timed_out_child()));
    }

    #[test]
    fn hash_fault_is_detected() {
        assert!(is_detected(&probe_changed_hash()));
    }
}
