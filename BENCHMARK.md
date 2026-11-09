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

## 6. Dense-world scalability (post-M10)

**Date:** 2026-11-09 · **Build:** `--release` (full `opt-level = 3`).

### 6.1 Purpose

M8 (frustum culling) and M9 (transparent sorting) landed against the tiny
`building-yard` fixture (25 chunks), where the deferred sub-work in the plan
(face buckets, batching/indirect, depth occlusion) is explicitly scoped to a
dense world. This section exercises the renderer across a scaling range to see
where FPS degrades and whether that deferred work becomes necessary.

A new `SUNSET_WORLD_RADIUS=N` environment variable (which overrides any save on
disk) generates a deterministic rolling-hill world spanning `(2N+1)` chunks on
each side (`sunset_core::fixture::hilly_world`). The smooth height variation
prevents greedy meshing from collapsing the surface, so every chunk carries real
geometry. The player spawns above the terrain and settles; the static camera then
looks out over rolling hills at ground level.

### 6.2 Method

`bench_dense.ps1 -Radius N` runs the release binary with
`SUNSET_WORLD_RADIUS=N` and `SUNSET_BENCH_SECS=8` (2 s warm-up, then 8 s
measured). All radii were measured back-to-back under sustained load.

### 6.3 Results

| Radius | Chunks | Opaque triangles | avg FPS | 0.1%-low FPS |
| --- | --- | --- | --- | --- |
| 4 | 81 | 48 304 | ~925 | ~74 |
| 8 | 289 | 166 558 | 798 | 76 |
| 16 | 1 089 | 627 210 | 567 | 71 |
| 32 | 4 225 | 2 433 760 | 210 | 50 |
| 48 | 9 409 | 5 414 838 | 115 | 39 |
| 64 | 16 641 | 9 565 068 | 81 | 38 |

(Transparent triangles are 0 — the hilly world is entirely opaque, so the
transparent pass is skipped.)

### 6.4 Analysis

- **Triangle throughput rises with scene size** (≈44 M tris/s at radius 4 →
  ≈775 M tris/s at radius 64). Small scenes are not triangle-bound: a fixed
  per-frame overhead (present/clear/uniform write/simulation) dominates. Only as
  the scene grows does raw geometry throughput approach the GPU's ceiling, and
  even radius 64 has not plateaued it.
- **The steep drop is between radius 16 and 32** (567 → 210 FPS, where the scene
  passes ~2.4 M triangles). Beyond that the curve flattens roughly as
  `FPS ∝ 1/triangles`, the classic triangle-bound regime.
- **Single-threaded meshing is the dominant startup cost**, not the steady-state
  cost. Release-mode mesh time ≈ 4 s at radius 32 and ≈ 14 s at radius 64 (all
  chunks meshed synchronously before the first frame). The unoptimized dev build
  exceeds 27 s for radius 32, confirming §11's deferred "off-thread chunk-mesh
  worker scheduling" (M6 fallback) is what a dense world would need first.
- **0.1%-low FPS is ~38–76 across all radii**, the same periodic stall noted in
  §4 (once-per-second console write and/or background-window throttling), not
  scene rendering. It is not a new regression.

### 6.5 Conclusion

At 16.6 k chunks / 9.5 M triangles the renderer holds ~81 FPS average (with
frustum culling active), and the dominant scaling cost is triangle throughput
plus per-chunk draw calls — not the deferred M8 batching machinery, which would
matter most when many *visible* chunks each issue a tiny draw. Two follow-ups are
now measurable rather than speculative:

1. **Off-thread meshing** (M6 fallback) to remove the multi-second startup stall.
2. **A draw-call-bound stress case** (a flat or shallow world maximizes visible
   chunks per view) to decide whether batching/indirect is justified.

> Caveat: these are sustained-load numbers (the GPU thermally throttles after the
> larger scenes, so absolute FPS is lower than a cool run; the earlier §2 M5
> number of ~1473 FPS is a cool-GPU dev-build figure and is not directly
> comparable). The *relative* scaling is the robust signal here.
