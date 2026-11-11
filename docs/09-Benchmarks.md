# Benchmark protocol

## Targets

REQ-03 is a MUST: **≥ 3,000 FPS** throughput on BENCH-01 in Fidelity at native resolution on the owner machine (PERF-T1, PERF-T2 in the [PRD](01-PRD.md)). 3,000 FPS is 333 µs per frame. 4,000 FPS (250 µs) is a STRETCH and is only reported. The owner's ~3,500 FPS report (≈ 286 µs) is context until M0 reproduces it with recorded settings; it becomes the PERF-T3 baseline.

M0 records CPU/topology/RAM/GPU/VRAM/backend/driver/OS/display/refresh/power plan/toolchain in [ENVIRONMENT](tracking/ENVIRONMENT.md). For the baseline client, record the version, exact loader/mod list/configs/JVM flags, packs, seed/save/camera/FOV, render and simulation distances, and entity load. Missing mod provenance makes the comparison incomplete, not invalid for PERF-T1.

## Scenes

| ID | Scene | Gate |
| --- | --- | --- |
| BENCH-00 | Minimal renderer floor: clear + HUD only | PERF-A14: > 20,000 FPS; never a gameplay claim |
| BENCH-01 | Owner scene as defined in PRD section 4.2 | **MUST ≥ 3,000 FPS** (M3+) |
| BENCH-01Q | BENCH-01 quick variant: 10 s warm-up, 30 s sample, 3 runs | Per-merge: ≥ 3,000 and ≤ 3% regression |
| BENCH-01S | Synthetic stand-in: deterministic terrain with the same visible-section count, vertex volume and pass mix as BENCH-01; no proprietary assets | M2: ≥ 4,000 FPS |
| BENCH-02 | Terrain flight at 8/16/32 chunks, 20 blocks/s | REQ-04 |
| BENCH-03 | Dense structures/chests/entities | Reported; budgets from M5 |
| BENCH-04 | Foliage/water/transparency | Reported; budgets from M3 |
| BENCH-05 | Redstone/entity simulation stress | PERF-T6 |
| BENCH-06 | Burst edits/teleport/dimension change | PERF-A07, no interval > 16.67 ms after the first frame |
| BENCH-07 | Pack reload/high-resolution textures | Reported |
| BENCH-08 | Enhanced LOD 32/64/128-chunk equivalent views | REQ-07 |
| BENCH-09 | Zero/10/100 no-op modules plus real workloads | REQ-10 |
| BENCH-10 | 60-minute soak and repeated unload | PERF-T7 |

Each scene has a manifest at `bench/scenes/<id>.toml` that freezes every parameter/count/path/hash (save hash, asset hash, camera path, settings, window, present mode request). Synthetic workloads do not substitute for BENCH-01 after M3. Compare Fidelity, vanilla and the pinned modded baseline at matched native resolution and visual load. Enhanced comparisons disclose scale/LOD/quality separately.

Manifest skeleton:

```toml
id = "BENCH-01"
profile = "fidelity"
save = { private_path = "fixtures/bench-01", sha256 = "<hex>" }
assets = { target_lock_sha256 = "<hex>" }
window = { width = 1920, height = 1080, mode = "borderless", gui_scale = 2 }
present = { request = ["immediate", "mailbox"], vsync = false, frame_cap = 0 }
settings = { render_distance = 8, simulation_distance = 8, fov = 70 }
timing = { warmup_s = 60, sample_s = 120, runs = 5 }

[[camera]]
kind = "hold"
yaw_deg = 180.0
pitch_deg = 0.0
seconds = 15.0
```

## Procedure

Use the pinned release build with its executable hash (PERF-A10). Uncapped, VSync off, effective present mode recorded; Fifo invalidates a gated run. Also measure capped ordinary play, latency and power separately. A normal monitor cannot display thousands of distinct frames per second; the gate measures engine throughput.

Platform notes: on Windows use borderless or exclusive fullscreen so the flip model can tear; windowed composition may cap throughput. On Linux Wayland compositors without tearing support offer no Immediate mode; use Mailbox or X11/exclusive and record it.

Full runs: warm 60 s, sample 120 s, five repetitions per configuration, alternating baseline/candidate order. Measure cold start and shader compilation separately. Record clocks/temperature/power where available, state unavailable sensors, avoid competing work (close browsers, set the high-performance power plan). Detect noisy or throttled runs and keep invalidated data with reasons; never discard slow runs selectively.

Throughput = completed rendered and presented frames / elapsed seconds, not mean instantaneous FPS. Record frame intervals into a preallocated ring (no allocation, PERF-A01). Separate CPU intervals, GPU duration, submit/present waits and tick compute. Timestamp collection must not serialize frames; measure instrumentation overhead (PERF-A12). Distinguish offscreen GPU throughput, engine render throughput and actual presentation. Frame generation and repeated cached frames do not count.

Report median run throughput, p50/p95/p99/p99.9/worst intervals, counts > 16.67/33.33/50 ms, tick tails, per-line frame-budget measurements (PRD 4.3), draw/dispatch/pass/submit counts, allocations per frame, resource creations per frame, mesh throughput, geometry counts, RAM/VRAM. Confidence intervals use independent runs or block resampling, not correlated individual frames.

## Gates

| Gate | Rule |
| --- | --- |
| PERF-T1 | Median of 5 runs ≥ 3,000 FPS and every run ≥ 2,850 FPS |
| PERF-T2 | p99 ≤ 0.50 ms, p99.9 ≤ 1.00 ms, zero intervals > 16.67 ms |
| Regression | Full runs: p95/p99 no more than 5% worse and steady-state memory no more than 5% higher than the accepted baseline. Quick runs: median throughput no more than 3% lower |
| Noise | M1 calibrates run-to-run noise. If the noise exceeds the regression threshold, the result is INCONCLUSIVE and reruns; inconclusive never counts as pass |
| Ticks | Ordinary 20 Hz loads: p99 tick computation < 50 ms; missed ticks reported |
| Modules | Zero-listener path ≤ 1% on BENCH-01; 100 no-op modules < 5% tick overhead on BENCH-09 |

An accepted baseline changes only by a reviewed commit with old and new numbers. Budget or threshold changes require an ADR. M1 sets cold-start/stream/reload deadlines from the baseline; missing deadlines block their release gates. Publish correctness and raw before/after results for every admitted optimization. Lower distance/particles/simulation are separate quality profiles, never hidden speed-ups.
