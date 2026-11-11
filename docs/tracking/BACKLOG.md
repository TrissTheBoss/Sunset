# Backlog M0–M3

Ordered, executable tasks for an LLM coding agent. Take the first task whose status is READY and whose dependencies are VERIFIED. One task per session. Status values follow [15](../15-Tracking-Rules.md). Expected exit codes follow [07](../07-Automated-Testing.md): 0 pass, 1 failure, 2 BLOCKED.

Common rules for every task: `#![forbid(unsafe_code)]`, no comments in production Rust/WGSL, no new crate unless the task names it, verify dependency APIs against the installed version, `cargo xtask check` exits 0 at handoff (from T-004 on), and PROGRESS/HANDOFF are updated with real commands and exit codes.

## M0 Reference and reset

### T-001 Repository adoption check
- Status: IMPLEMENTED · REQ-16 · Depends: none
- Scope: read-only Git inspection; write `docs/tracking/PROGRESS.md` entry only.
- Do: record default branch, HEAD, local changes, open branches/tags; confirm the handbook tree is the active tree or record that the owner must perform [13](../13-GitHub-Wiki.md).
- Accept: PROGRESS entry with the commands used. No Git writes.
- Out of scope: merging, pushing, tagging.

### T-002 Workspace skeleton and toolchain pin
- Status: IMPLEMENTED · REQ-01, REQ-24 · Depends: T-001
- Scope: `rust-toolchain.toml`, root `Cargo.toml` (workspace, `[workspace.lints]` with `unsafe_code = "forbid"`, release profile per PERF-A10), `.cargo/config.toml` alias `xtask = "run --package xtask --"`, `xtask/` crate with `clap` CLI and a `doctor` subcommand stub that returns exit 2 with "NOT IMPLEMENTED", `.gitignore` (`target/`, `runs/`, `private/`).
- Accept: `cargo build` → 0; `cargo xtask doctor --non-interactive` → 2 with the message; `cargo clippy --all-targets -- -D warnings` → 0.
- Out of scope: game crates.

### T-003 Doctor and result schema v1
- Status: IMPLEMENTED · REQ-18, REQ-12 · Depends: T-002
- Scope: `xtask` only.
- Do: detect OS, CPU model/cores, RAM, Rust toolchain, GPU adapters/backends through `wgpu` adapter enumeration, release-profile settings (PERF-A10), private root presence (`SUNSET_PRIVATE_ROOT`, default `../sunset-private`), local launcher install (read-only). Write `runs/<run-id>/doctor.json` (schema version 1, fields per [07](../07-Automated-Testing.md)) and a Markdown summary. Exit 0 when all required checks pass, 2 when a prerequisite is missing, 1 when a check fails.
- Accept: unit tests for exit-code mapping and schema serialization; doctor on a machine without the game install → 2 with an actionable message; results are reproducible apart from timestamps.
- Out of scope: reference hashing (T-005).

### T-004 Source lint and `xtask check`
- Status: IMPLEMENTED · REQ-17, REQ-01 · Depends: T-002
- Scope: `xtask lint`, `xtask check`.
- Do: token-aware comment detection for Rust (a lexer crate such as `ra_ap_rustc_lexer`, verified) and a small hand-written WGSL lexer (line comments, nested block comments, strings do not exist in WGSL). Exempt test modules/files and license headers per AGENTS.md. Check that `#![forbid(unsafe_code)]` is effective in every first-party crate. `check` = fmt check + clippy `-D warnings` + lint + unit tests.
- Accept: fault fixtures (comment in a Rust string must pass, real comment must fail, nested WGSL block comment must fail, license header must pass); `cargo xtask check --non-interactive` → 0 on the clean tree, → 1 on an injected comment.
- Out of scope: CI configuration.

### T-005 Reference lock
- Status: READY · REQ-12, REQ-02 · Depends: T-003
- Do: locate the pinned target version in the local launcher directory (read-only), hash client/server jars and asset index, read protocol/pack versions from version metadata, write private `target-lock.json` and committed `target-lock.public.json` per [02](../02-Parity.md). Missing install → 2.
- Accept: hashing tested on synthetic fixture files; a real run either produces the lock or a BLOCKED report; nothing proprietary is committed (`git status` review).

### T-006 Inventory via data generator
- Status: READY · REQ-02 · Depends: T-005
- Do: `cargo xtask inventory` runs the pinned server jar's report generator into the private root with a timeout, parses the report files that exist, and writes counts per parity domain into `docs/tracking/PARITY.md`. Missing Java or jar → 2.
- Accept: parser tests on small synthetic report fixtures; real counts recorded or BLOCKED.

### T-007 Environment and owner baseline
- Status: READY · REQ-03 · Depends: T-003
- Do: fill [ENVIRONMENT](ENVIRONMENT.md) from the doctor output. Write `bench/scenes/bench-01.toml` per PRD 4.2. Produce owner instructions for creating the `bench-01` save in vanilla and for measuring the baseline client on it; record results when the owner provides them, else mark PERF-T3 BLOCKED (owner action).
- Accept: manifest parses (test); ENVIRONMENT has no silent blanks (unknown = `UNKNOWN`).

## M1 Measurement before speed

### T-101 `sunset-metrics` crate
- Status: NOT STARTED · REQ-03, REQ-04 · Depends: T-004
- Do: fixed-capacity interval ring (no allocation after construction), throughput as frames/elapsed, exact percentile computation on a sorted copy outside the hot path, counters (draws, passes, submits, resource creations, uploads), and a counting global allocator with a per-thread frame scope.
- Accept: property tests for percentiles against a naive reference; test that recording 1,000,000 intervals allocates 0 bytes.

### T-102 Runner core
- Status: NOT STARTED · REQ-18 · Depends: T-003
- Do: child processes with argument arrays, timeouts, process-group cleanup, immutable `runs/<run-id>/`, JSON/JUnit/Markdown output, `cargo xtask report --run`.
- Accept: fault tests: hanging child → 4, crashing child → 3, missing prerequisite → 2; no orphan processes after each.

### T-103 Bench harness
- Status: NOT STARTED · REQ-03 · Depends: T-101, T-102
- Do: `cargo xtask bench --suite <id> [--gate]` reads `bench/scenes/<id>.toml`, launches the client binary with the manifest, collects the metrics JSON, runs repetitions in alternating order, flags noisy/throttled runs, and applies the gates from [09](../09-Benchmarks.md).
- Accept: synthetic fake client tests: a fake at 2,900 FPS fails PERF-T1 (exit 1), at 3,100 passes, a noisy fake → INCONCLUSIVE.

### T-104 Harness self-tests
- Status: NOT STARTED · REQ-18 · Depends: T-103
- Do: injected faults from [07](../07-Automated-Testing.md) that exist so far (allocation in the frame scope, 2 ms stall, timed-out child, changed hash); each must fail for the intended reason.
- Accept: `cargo xtask test --suite harness` → 0 with every fault detected.

## M2 Renderer core

### T-201 Device, window and frame loop
- Status: NOT STARTED · REQ-03 · Depends: T-101
- Crates: `sunset-render`, `sunset-client` (binary).
- Do: winit window, wgpu instance/adapter/device with validation on in debug and off in release (PERF-A11), present-mode selection Immediate → Mailbox → Fifo with the effective mode logged once, continuous redraw (PERF-A14), resize/minimize, device-loss recovery, 2-slot frame ring (PERF-A04), one submit per frame, HUD clear pass. Emits metrics JSON when launched by the harness.
- Accept: BENCH-00 > 20,000 FPS on the owner machine (or recorded value plus analysis); 0 allocations per frame; 0 resource creations per frame after warm-up.

### T-202 GPU timing ring
- Status: NOT STARTED · Depends: T-201
- Do: timestamp queries per pass when supported, resolved into a ring and read 2 frames later (PERF-A12); per-line CPU tracing spans for the PRD 4.3 budget table.
- Accept: on/off paired run shows ≤ 2% overhead; graceful absence when timestamps are unsupported.

### T-203 `sunset-core` basics
- Status: NOT STARTED · REQ-02 · Depends: T-004
- Do: block/section/chunk coordinates with vanilla floor-division semantics for negatives, dense block-state handles, paletted section container with the target's bits-per-entry rules, Java-semantics integer helpers.
- Accept: property tests for round trips; vectors for negative coordinates and extreme integers.

### T-204 Synthetic terrain and BENCH-01S
- Status: NOT STARTED · REQ-03 · Depends: T-203
- Do: deterministic generator (own noise, not vanilla worldgen) producing hills, caves, overhangs, foliage-like cutout and water so that the visible-section count and vertex volume approximate BENCH-01 (document the calibration in `bench/scenes/bench-01s.toml`).
- Accept: generation is seed-deterministic (hash test).

### T-205 Reference mesher and packed vertex format
- Status: NOT STARTED · REQ-06 · Depends: T-203
- Do: culled-face CPU mesher for full cubes with neighbor-section awareness; vertex layout ≤ 16 bytes (ADR-0005 amendment with layout table).
- Accept: unit tests on hand-built sections (single block, slab of blocks, section borders).

### T-206 Greedy mesher with equivalence proof
- Status: NOT STARTED · REQ-06 · Depends: T-205
- Do: greedy merging only under the equivalence rules of [04](../04-Renderer.md); a CPU raster/ID comparison against the reference mesher.
- Accept: property test over random sections: raster-equal output; triangle-reduction statistic reported on BENCH-01S.

### T-207 Geometry arena and staging ring
- Status: NOT STARTED · REQ-03 · Depends: T-201, T-205
- Do: fixed-size GPU buffers with a free-list sub-allocator, a staging ring that respects in-flight frames, per-frame upload budget (PERF-A07).
- Accept: allocator property tests (no overlap, full reuse); no write to an in-flight region (debug assertion test).

### T-208 Culling and indirect draws
- Status: NOT STARTED · REQ-03, REQ-06 · Depends: T-207
- Do: section frustum culling with conservative bounds, contiguous indirect-draw list per pass, multi-draw-indirect when the pinned wgpu supports it on the backend, else a looped indirect fallback; a direct-draw reference path behind a debug toggle.
- Accept: zero false culling on the visibility suite; toggle equivalence (identical ID buffer); BENCH-01S terrain draws ≤ 64.

### T-209 Terrain shading
- Status: NOT STARTED · Depends: T-208
- Do: WGSL opaque/cutout/translucent pipelines with a texture array, lightmap, AO, fog using synthetic textures; translucent section sorting.
- Accept: debug-buffer checks against analytic expectations ([08](../08-Visual-QA.md) comparator 2).

### T-210 Job system
- Status: NOT STARTED · REQ-23 · Depends: T-203
- Crate: `sunset-jobs`.
- Do: worker pool (cores − 2), priority queue, revisions, stale-result rejection, cancellation, bounded queues.
- Accept: controlled-scheduling tests for stale rejection and cancellation; no unbounded growth under flood.

### T-211 HUD batch and synthetic font
- Status: NOT STARTED · REQ-19 · Depends: T-201
- Accept: one draw per material class; 0 allocations per frame with changing text.

### T-212 M2 performance gate
- Status: NOT STARTED · REQ-03 · Depends: T-202, T-206, T-208, T-209, T-210, T-211
- Do: run BENCH-01S full; fill the budget table measurements; profile and fix until the gate passes or an ADR documents the bottleneck.
- Accept: `cargo xtask bench --suite bench-01s --gate` → 0 at ≥ 4,000 FPS; PERF-A01–A15 counters green.

## M3 Foundation integration (VS-1)

Status of all M3 tasks: NOT STARTED. They are outlines; refine each into a full task block (scope, accept commands, out of scope) at the M2 exit, before starting it.

### T-301 Anvil region reader
- REQ-20, REQ-21 · Depends: T-203. Bounded read-only NBT/region parser into paletted sections; fuzz target; private `bench-01` save loads.

### T-302 Asset resolver
- REQ-12 · Depends: T-005. Read-only local assets: blockstates, models with inheritance, textures, animation metadata; texture array/atlas build; derived cache with hashes.

### T-303 Model baker
- REQ-02 · Depends: T-302, T-205. Non-cube models, multipart, UV rotation/lock, tint index with biome colormaps, cutout/translucent classification.

### T-304 Fidelity lighting, fog, sky, clouds, water
- REQ-02 · Depends: T-303, T-209. Stored light from the save, smooth lighting/AO, target fog/sky colors; VIS-BENCH-01 comparison.

### T-305 Tick thread and snapshot handoff
- REQ-03 · Depends: T-210. Crate `sunset-sim` skeleton; 20 Hz tick thread; lock-free triple buffer; interpolation; contention counters at zero (PERF-A02).

### T-306 Settings framework and Video screen
- REQ-19 · Depends: T-211. Stable keys/types/defaults; render distance, VSync, frame cap; 15 s display rollback.

### T-307 Protocol/session probe and storage transaction design
- REQ-11, REQ-20 · Depends: T-301. Narrow handshake/status codec probe; ADR for write transactions. No gameplay.

### T-308 VS-1 gate
- REQ-03, REQ-05 · Depends: T-301…T-307. `cargo xtask bench --suite bench-01 --gate` → 0 (PERF-T1, PERF-T2); VIS-BENCH-01 within calibrated thresholds; foundation gate items in [10](../10-Roadmap.md) demonstrated; then write M4 tasks into this backlog.
