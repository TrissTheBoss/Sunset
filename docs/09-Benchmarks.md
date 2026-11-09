# Benchmark protocol

The owner's ~3,500 FPS claim is context until reproduced. That is ~0.286 ms/frame; 4,000 FPS is 0.250 ms, about 12.5% less frame interval. Hardware/baseline settings are unknown.

M0 records CPU/topology/RAM/GPU/VRAM/backend/driver/OS/display/refresh/power/toolchain. Record comparison client version, exact loader/mod/config/JVM flags, packs, seed/snapshot/camera/FOV/render and simulation distances/entity load. Missing mod provenance means incomplete comparison.

| ID | Scene |
| --- | --- |
| BENCH-00 | Minimal renderer floor; never gameplay claim |
| BENCH-01 | Matched owner scene/camera; 4,000 FPS stretch |
| BENCH-02 | Terrain flight at 8/16/32 chunks |
| BENCH-03 | Dense structures/chests/entities |
| BENCH-04 | Foliage/water/transparency |
| BENCH-05 | Redstone/entity simulation stress |
| BENCH-06 | Burst edits/teleport/dimension change |
| BENCH-07 | Pack reload/high-resolution textures |
| BENCH-08 | Enhanced LOD 32/64/128-chunk equivalent views |
| BENCH-09 | Zero/10/100 no-op modules plus real workloads |
| BENCH-10 | 60-minute soak and repeated unload |

Manifests freeze every parameter/count/path/hash. Synthetic provisional workloads do not substitute for BENCH-01. Compare Fidelity, vanilla and pinned modded client at matched native resolution and visual load. Enhanced comparisons disclose scale/LOD/quality separately.

## Procedure

Pinned release build and executable hash. Uncap/VSync-off for throughput, record actual present mode. Also measure capped ordinary play/latency/power. Normal monitor refresh cannot display thousands of distinct frames per second.

Warm 60 seconds, sample 120 seconds, five repetitions/configuration, alternate baseline/candidate order. Cold start/compilation separate. Record clocks/temperature/power when available, state unavailable sensors, avoid competing work. Detect noisy/throttled runs and retain invalidated data with reasons; never discard slow runs selectively.

Throughput = completed rendered frames / elapsed seconds, not mean instantaneous FPS. Separate CPU intervals, GPU duration, submit/present waits and tick compute. Async timestamp collection must not serialize frames; measure instrumentation overhead. Distinguish offscreen GPU throughput, engine render throughput and actual presentation. Frame generation/repeated cached frames do not count.

Report median run throughput, p50/p95/p99/p99.9/worst intervals, counts >16.67/33.33/50 ms, tick tails, mesh throughput, geometry/draw counts, RAM/VRAM/allocations. Confidence intervals use independent runs or block resampling, not correlated individual frames as independent samples.

## Gates

BENCH-01 stretch: median five-run throughput >=4,000 actual rendered FPS at matched native settings; show interval and whether every run passed. This proves only the named workload.

Proposed regression budget: p95/p99 no worse than 5% and steady-state memory no more than 5% growth against accepted baseline. Calibrate noise; inconclusive means rerun, not victory. Budget changes require ADR.

Ordinary 20 Hz target loads: p99 tick computation <50 ms and missed ticks reported. Initial synthetic presentation target: p99 <8.33 ms on named owner tier, to be validated rather than asserted achieved. Zero-listener path should have no observable regression; 100 no-op modules target <5% tick overhead in BENCH-09.

M1 establishes explicit cold-start/stream/reload deadlines from baseline and product needs. Missing deadlines block their release gates. Publish correctness and raw before/after results for every admitted optimization. Lower distance/particles/simulation are separate quality profiles, never hidden speedups.
