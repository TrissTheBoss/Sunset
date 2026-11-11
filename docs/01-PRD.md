# Product requirements

Revision 2 · 10 November 2026 · Owner decisions recorded in [ADR-0002](decisions/ADR-0002-performance-target.md) and [ADR-0004](decisions/ADR-0004-shader-pack-scope.md).

## 1. Product goal

Sunset is an original, memory-safe reimplementation of the Minecraft Java Edition client, integrated server and dedicated server, written in safe Rust with custom WGSL shaders. It has two product pillars, in this priority order:

1. **Performance (MUST):** at least 3,000 rendered frames per second on the defined benchmark scene BENCH-01, measured continuously from M2 onward. A regression below the gate blocks the merge.
2. **Fidelity (MUST):** observable behavior of the pinned target version, demonstrated by differential evidence against local vanilla.

Everything else (LOD, upscaling, modding, packs, shader packs) is built on these two pillars and may never push either below its gate. When a feature conflicts with a gate, the gate wins and the feature waits or is redesigned.

Target: Java Edition 26.3, provisional (see [ADR-0003](decisions/ADR-0003-reference-target.md)). If M0 cannot verify that reference, the work stops as BLOCKED and an explicit decision follows; nobody switches versions silently.

## 2. Priority vocabulary

| Priority | Meaning |
| --- | --- |
| MUST | Release-blocking. The milestone named in the requirement cannot exit without its evidence. |
| SHOULD | Planned for 1.0. Deferral requires an ADR. |
| COULD | Started only after its milestone, and only while all MUST gates are green. |
| WON'T (now) | Out of scope. Reopening requires an ADR. See section 8. |
| STRETCH | Measured and reported, never a gate. |

## 3. Profiles

| Profile | Rules |
| --- | --- |
| Vanilla Fidelity | Native resolution; target geometry/light/fog/materials; no visual enhancements. **The profile that REQ-03 is measured in.** |
| Enhanced (proposed default for play) | Identical simulation; distant LOD; Auto upscaling with native fallback. Must never be slower than Fidelity on BENCH-01 and BENCH-02 (PERF-T5). |
| Modded | Explicit module graph and isolated saves; changed behavior clearly labeled. |

Compatibility Session is a policy layered over either render profile: legitimate account, matching protocol, server authority, no gameplay-changing local modules. Cosmetic rendering does not prove compatibility. Strict visual parity applies only to Fidelity.

## 4. Performance contract

### 4.1 Targets

| ID | Priority | Target | Evidence |
| --- | --- | --- | --- |
| PERF-T1 | MUST | Median of 5 runs ≥ **3,000 FPS** throughput on BENCH-01 in Fidelity at native resolution; every individual run ≥ 2,850 FPS | `cargo xtask bench --suite bench-01 --gate` |
| PERF-T2 | MUST | Frame interval p99 ≤ 0.50 ms, p99.9 ≤ 1.00 ms, zero intervals > 16.67 ms in the sampling window | Same run |
| PERF-T3 | SHOULD | Throughput ≥ 1.0× the owner's baseline client on BENCH-01 (baseline recorded in M0) | Matched comparison per [09](09-Benchmarks.md) |
| PERF-T4 | STRETCH | ≥ 4,000 FPS on BENCH-01 | Same run, reported |
| PERF-T5 | MUST | Enhanced throughput ≥ Fidelity throughput on BENCH-01 and BENCH-02 | Paired runs |
| PERF-T6 | MUST | 20 Hz tick p99 < 50 ms on BENCH-05; the tick thread never blocks the render thread | Tick metrics plus contention counters |
| PERF-T7 | MUST | Steady-state memory growth ≤ 1% over the 60-minute BENCH-10 soak | Soak report |

Throughput means completed rendered and presented frames divided by elapsed seconds. Frame generation, repeated frames and offscreen-only rendering do not count.

### 4.2 BENCH-01 definition

These defaults apply until M0 records the owner's original ~3,500 FPS run. If that run used different settings, ADR-0002 is amended before the M2 gate and both definitions remain recorded.

| Parameter | Value |
| --- | --- |
| Hardware | Owner machine, recorded in [ENVIRONMENT](tracking/ENVIRONMENT.md) during M0 |
| Backend | DX12 on Windows, Vulkan on Linux; the actual adapter/driver is recorded |
| Build | Shipped release profile (PERF-A10), executable hash recorded |
| Window | 1920×1080 borderless fullscreen, render scale 100%, GUI scale 2, hotbar and crosshair visible, debug overlay hidden |
| Present | Immediate if the surface offers it, otherwise Mailbox; the actual mode is recorded; VSync off; frame cap off |
| World | Private fixture save `bench-01`, created in vanilla target with seed `sunset`, default world type, pre-generated radius 16 chunks around spawn, terrain/foliage/water only, all entities removed, no block entities in view |
| Game rules | Daylight cycle off at time 6000, weather cycle off and clear, mob spawning off, other rules default |
| Settings | Target defaults except render distance 8, simulation distance 8, FOV 70 |
| Camera | Spectator at spawn, eye 1.62 above the surface: 4 × 15 s fixed headings N/E/S/W at pitch 0, then a 60 s constant 360° yaw sweep. Defined in `bench/scenes/bench-01.toml` |
| Timing | Warm-up 60 s, sampling 120 s, 5 runs, alternating with the baseline when comparing |

BENCH-01Q is the per-merge quick variant: identical scene, 10 s warm-up, 30 s sampling, 3 runs. It fails on median < 3,000 FPS or a regression of more than 3% against the accepted baseline.

### 4.3 Frame budget

With two frames in flight, throughput is limited by the slowest pipeline stage, not by the sum of stages. Each stage must fit within 333 µs. Planning uses 25% headroom: **250 µs per stage**.

| CPU render thread | Budget (µs) | GPU | Budget (µs) |
| --- | --- | --- | --- |
| Input/events/window | 10 | Opaque terrain | 120 |
| Snapshot acquire + interpolation | 10 | Cutout terrain | 35 |
| Section visibility + indirect list | 50 | Translucent terrain/water | 35 |
| Entities/block entities/particles prepare | 40 | Entities/block entities/particles | 20 |
| HUD/UI build | 25 | Sky/clouds/fog | 15 |
| Upload scheduling (bounded) | 25 | HUD/UI | 10 |
| Command encoding + single submit | 50 | Reserve | 15 |
| Acquire/present | 30 | | |
| Reserve | 10 | | |
| **Total** | **250** | **Total** | **250** |

The budgets are planning values. Tracing spans and GPU timestamps measure each line. Exceeding one line flags a regression for review; exceeding a stage total fails the gate. Budget changes require an ADR.

### 4.4 Performance architecture requirements

These are MUST requirements from M2 onward. Each has an automated check.

| ID | Requirement | Verification |
| --- | --- | --- |
| PERF-A01 | Zero heap allocations on the render thread per frame in steady state | Counting allocator in bench builds; test asserts 0 over 10,000 frames |
| PERF-A02 | The render thread never takes a lock shared with simulation or workers; snapshots are handed over lock-free (triple buffer or atomic swap) | Code review rule plus contention counters at zero |
| PERF-A03 | The render thread blocks only in surface acquire/present; no synchronous buffer map or wait-poll inside the frame loop | Tracing span audit |
| PERF-A04 | At most 2 frames in flight; per-frame resources are indexed by ring slot | Unit test plus GPU validation run |
| PERF-A05 | Terrain is drawn from sub-allocated geometry arenas through indirect draws. BENCH-01 limits: ≤ 64 terrain draw calls, ≤ 300 draw/dispatch calls in total, ≤ 8 render passes, exactly 1 queue submit per frame | Per-frame counters in the bench report |
| PERF-A06 | No per-frame creation of buffers, textures, bind groups, pipelines or shader modules. All pipelines are built at load or reload | Creation counters assert zero in steady state |
| PERF-A07 | Uploads per frame are capped (default 2 MiB and 25 µs CPU); excess waits in a queue | Bench counter plus BENCH-06 |
| PERF-A08 | Worker pool size = logical cores − 2 (minimum 1); render and tick threads are reserved | Environment report plus thread census |
| PERF-A09 | The hot path contains no log formatting, string-keyed lookups or per-object dynamic dispatch; it uses dense handles | Review rule plus clippy configuration |
| PERF-A10 | Release profile: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"` for the game binary, `debug = "line-tables-only"`. Baseline CPU target x86-64-v2; higher tiers only by ADR with measurements | `Cargo.toml` check in `xtask doctor` |
| PERF-A11 | GPU validation and debug labels are off in release builds and on in debug/test builds | Doctor reports the effective flags |
| PERF-A12 | GPU timestamp queries are resolved asynchronously into a ring and read at least 2 frames later; instrumentation overhead ≤ 2% | Paired run with instrumentation on/off |
| PERF-A13 | Fidelity adds no post-processing pass that the target does not have; HUD/UI is one batched pass with one atlas | Pass counter |
| PERF-A14 | The uncapped event loop redraws continuously without sleep; capped mode uses precise waiting | BENCH-00 floor > 20,000 FPS on the owner machine |
| PERF-A15 | Every change touching render, simulation, assets or threading runs BENCH-01Q (BENCH-01S before M3) on the trusted GPU runner | CI lane plus PROGRESS evidence |

### 4.5 Performance gates per milestone

| Milestone | Gate |
| --- | --- |
| M1 | BENCH-00 harness works; measurement noise on the owner machine is calibrated and recorded |
| M2 | Synthetic BENCH-01S (same visible-section count and vertex volume as BENCH-01) ≥ **4,000 FPS**. This is 33% headroom for later real content |
| M3 (VS-1) | Real BENCH-01 ≥ 3,000 FPS with the real save and local assets, plus PERF-T2 |
| M4–M8 | BENCH-01Q ≥ 3,000 FPS on every merge; BENCH-01 full at each milestone exit |

## 5. First vertical slice VS-1 (M3 exit)

VS-1 is the first deliverable that proves both pillars together. It contains:

- Read-only loading of the `bench-01` Anvil region files (bounded, fuzzed parser) into paletted sections.
- Block models, blockstates and textures resolved from the local legally installed target assets, including non-cube models, multipart, UV rotation and biome tint.
- Fidelity terrain rendering with stored sky/block light, smooth lighting/AO, fog, sky, clouds, water and foliage. No light engine yet.
- Spectator fly camera, hotbar and crosshair HUD, and a minimal Video settings screen (render distance, VSync, frame cap).
- A tick thread with trivial simulation and lock-free snapshot handoff, which proves PERF-A02 before real simulation exists.
- `cargo xtask bench --suite bench-01 --gate` passing PERF-T1 and PERF-T2.
- `cargo xtask visual --case VIS-BENCH-01` within calibrated thresholds against a vanilla capture of the same camera.

VS-1 explicitly excludes gameplay, worldgen, networking, entities and a light engine.

## 6. Requirements

| ID | Priority | Milestone | Requirement | Acceptance evidence | Tests |
| --- | --- | --- | --- | --- | --- |
| REQ-01 | MUST | M0, continuous | Safe Rust runtime, custom WGSL, no first-party C/C++ | `#![forbid(unsafe_code)]` in every first-party crate; dependency unsafe/native audit; no first-party C/C++ sources | T-LINT-UNSAFE, T-AUDIT-DEPS |
| REQ-02 | MUST | M4–M7 (phased) | Complete observable target parity | Inventory mapped to tests per [02](02-Parity.md); no unresolved critical mismatch | T-PAR-* |
| REQ-03 | MUST | M2 synthetic, M3 real | ≥ 3,000 FPS on BENCH-01 (section 4) | PERF-T1, PERF-T2 | BENCH-01, BENCH-01Q, BENCH-01S |
| REQ-04 | MUST | M3, continuous | Smooth performance | PERF-T2, PERF-T6, PERF-T7; BENCH-02 at 16 chunks and 20 blocks/s: p99 ≤ 2 ms, new sections visible ≤ 1 s after entering range | BENCH-02, BENCH-05, BENCH-10 |
| REQ-05 | MUST | M3 | Foundation before mass content | Foundation gate in [10](10-Roadmap.md) | All M3 exit checks |
| REQ-06 | MUST | M2 | Culling and greedy meshing | Zero false culling in the visibility suite; greedy output raster-equal to the reference mesher; ≥ 40% fewer terrain triangles than culled-face meshing on BENCH-01 (planning target) | T-VIS-CULL-*, T-MESH-EQ-* |
| REQ-07 | SHOULD | M2 prototype, M8 final | Attractive fast LOD | No cracks/false walls; BENCH-08 budgets; invalidation tests | BENCH-08, VIS-LOD-* |
| REQ-08 | SHOULD | M2 prototype, M8 final | Standard upscaling | Off/Auto/fixed scales, native HUD, quality metrics per [08](08-Visual-QA.md) | VIS-UPSCALE-* |
| REQ-09 | SHOULD | M6 | Every game domain extensible | Surface inventory and independent example modules per [06](06-Modding-Packs.md) | T-MOD-SURFACE-* |
| REQ-10 | SHOULD | M6 | Own mod platform | Versioned contracts and bounds; zero-listener path ≤ 1% on BENCH-01; 100 no-op modules ≤ 5% tick overhead | BENCH-09, T-MOD-* |
| REQ-11 | MUST | M6 | Vanilla-server interoperability | Authorized protocol/state/authentication matrix | T-NET-* |
| REQ-12 | MUST | M0 | Legal local resources | Read-only importer, provenance, clean distribution | T-ASSET-*, release audit |
| REQ-13 | MUST | M5–M6 | Resource and data packs | Pinned format, semantics, precedence and reload tests | T-PACK-* |
| REQ-14a | SHOULD | M6 | Native Sunset WGSL shader packs | Typed pass/resource/settings API and examples | T-SHADERPACK-* |
| REQ-14b | COULD | After M6 | Iris/OptiFine-format shader pack compatibility | Separate feasibility ADR, then a coverage matrix | — |
| REQ-15 | MUST | M5 | Commands and admin content | Grammar, permissions, completion, command blocks, structures, full target inventory | T-PAR-CMD-* |
| REQ-16 | SHOULD | M0 repo, M8 Wiki | GitHub/Wiki | Recoverable replacement, release history, versioned published documentation | Wiki publish task |
| REQ-17 | MUST | Continuous | Clean production source | No first-party production comments or dead code; licenses preserved | T-LINT-SOURCE |
| REQ-18 | MUST | M1 | Unattended verification | Deterministic runner with meaningful BLOCKED/failure statuses and fault injection per [07](07-Automated-Testing.md) | T-HARNESS-* |
| REQ-19 | MUST | M3 framework, M7 complete | Vanilla-style settings | Familiar navigation, local UI assets, engine controls | T-UI-SETTINGS-* |
| REQ-20 | MUST | M4 | Save integrity | Compatible semantics, backups, atomic writes, migration/recovery | T-SAVE-* |
| REQ-21 | MUST | Continuous | Security and privacy | Bounded parsers, sandboxed modules, secure tokens, no telemetry by default | T-FUZZ-*, T-SEC-* |
| REQ-22 | MUST | M7 | Accessibility and localization | Scaling, rebinding, subtitles, fonts/languages, narration | T-UI-A11Y-* |
| REQ-23 | MUST | M3 | Resilience | Device/network loss, low memory, missing assets, failed reload | T-RESIL-* |
| REQ-24 | MUST | M8 | Reproducible distribution | Pinned build, SBOM/licenses/checksums, rollback | Release checklist |

## 7. Scope notes

Full parity scope includes obscure states, scoreboard/command return values, world border, random streams, upgrades and chat/security behavior, not only movement and building. Preview releases list supported features; they do not complete REQ-02. Publish raw verified/required counts and gaps. Use "no known differences in declared scope" with evidence instead of claiming proof of 100%.

Added improvements, each requiring tests: private secret-stripped reproduction bundles, per-mod performance attribution, crash-safe saves, provenance tracking, accessible diagnostics, and quality-preserving fallback paths.

## 8. Non-goals (WON'T now)

- Bedrock Edition.
- Executing existing Java mod binaries (Fabric, Forge, NeoForge).
- Iris/OptiFine shader pack compatibility before M6 (REQ-14b is COULD afterwards).
- Multiple Java versions before 1.0.
- A guarantee of acceptance by every third-party anticheat.
- Frame generation, hardware ray tracing, mesh shaders and variable-rate shading.
- Mobile, console and web targets.
- Bundled Mojang assets of any kind.
- Telemetry enabled by default.

## 9. Platforms

| Tier | Platform | Gates |
| --- | --- | --- |
| 1 | Owner machine (OS recorded in M0) | All MUST gates including PERF-T1 |
| 1 | Windows 10/11 x64 with DX12, Linux x64 with Vulkan | Functional plus performance reporting |
| 2 | macOS with Metal | Functional; performance reported, not gated |

Minimum GPU capabilities are fixed in M2 by ADR-0005 after capability tests (indirect drawing, texture arrays, timestamp queries optional).

## 10. Prohibitions

No gameplay simplification for FPS, anticheat evasion, authentication bypass, proprietary redistribution or adopted complete WGSL renderer. Reuse audited Rust cryptography/compression/platform primitives. No general scripting runtime in per-voxel hot paths. Lower distances, particles or simulation are separate visible quality settings, never hidden speed-ups inside a gated measurement.
