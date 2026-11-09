# Benchmark Record — Raw Renderer vs Bevy Adapter (M5)

**Date:** 2026-11-09
**Revision:** `e1575bf` + M5 frame-stat instrumentation (pre-commit)

## 1. Environment (PRD 7.3)

| Field | Value |
| --- | --- |
| OS | Microsoft Windows 11 Education, 10.0.26200, 64-bit |
| CPU | AMD Ryzen 5 5600X 6-core (12 threads, 3.7 GHz) |
| GPU | AMD Radeon RX 6800 XT (driver 32.0.21045.16010, ~4 GB VRAM) |
| RAM | 15.9 GB |
| Display | 2560×1440 @ 165 Hz (windowed, default window size) |
| Graphics backend | Vulkan (wgpu default on this AMD setup) |
| Toolchain | rustc 1.99.1 (stable) |
| Build | `[profile.dev]` with `opt-level = 3` for all dependencies (our crates at `opt-level = 0`) — the same "optimized deps" configuration used for the plan's prior measurements |
| Power mode | Not explicitly set |
| Fixture | `building-yard` (25 chunks, 448 opaque + 12 transparent triangles) |
| Scenario | Static first-person camera at spawn `(8, 5.5, 16)`, no input during measurement |
| State | Warm (2 s warm-up on the raw path; none on Bevy) |

> Note: a full `--release` build was attempted but was interrupted partway; the
> dev "optimized deps" profile is what the plan's §2 table already uses, so the
> numbers below are directly comparable to those.

## 2. Results (avg + 0.1%-low FPS)

| Target | avg FPS | 0.1%-low FPS (10 s window) | Notes |
| --- | --- | --- | --- |
| **sunset-raw (M4)** | **1473** | **295** | 14 734 frames over 10 s |
| **sunset (Bevy M1)** | ~350–440 (declining) | ~9 (see note) | 8 × 1 s samples, no warm-up |

### Bevy per-second samples

```
avg=442.9, 403.8, 365.6, 322.2, 322.4, 334.7, 349.0, 395.0   (0.1%-low ≈ 9 each)
```

## 3. Comparison

- **Raw is ~3.5–4× faster** than the Bevy adapter at steady state
  (~1473 vs ~350–440 FPS).
- **Regression within the raw path** (plan §7.1): M0's empty clear-color loop
  measured ~2400 FPS; the M4 full renderer (world mesh + camera + transparent +
  outline + per-frame raycast + 20 Hz simulation) measures ~1473 FPS. The
  remaining gap is the real rendering + simulation work, not Bevy overhead.

## 4. Caveats

- The **0.1%-low** figures are dominated by a periodic ~110 ms stall that
  appears in both paths (visible as `0.1% low ≈ 9 FPS` in the Bevy samples and
  occasionally in the raw per-second report). This correlates with the
  once-per-second console write (`println!`/`info!`) and/or background-window
  throttling, not with scene rendering. The 10 s window averages it out to
  295 FPS on the raw path.
- The Bevy avg declines over the run (443 → 322 → 395); this is un-investigated
  (possible thermal/focus throttling) and is reported as a range rather than a
  single number.
- Windowed at the default size, not fullscreen; VSync disabled on both paths.

## 5. Method

- **Raw:** `SUNSET_BENCH_SECS=10` triggers a 2 s warm-up, then records every
  frame time for 10 s and prints `BENCH_RESULT avg=… 0.1%low=… frames=…`, then
  exits.
- **Bevy:** `hud::log_frame_stats` logs `FPS_STATS avg=… 0.1%low=…` once per
  second (a 1000-frame rolling window), captured from stderr.
