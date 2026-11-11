# Progress

Current milestone: **M2 IN PROGRESS** (M0/M1 code complete; M0 exit blocked on server jar + owner baseline). Current task: T-201 done (next: [T-202](BACKLOG.md#t-202-gpu-timing-ring)).

Implementation baseline: fresh rewrite. Results of the previous prototype are not inherited; never carry "M2 complete" from the old implementation into this rewrite.

## Milestone status

| Milestone | Status | Evidence |
| --- | --- | --- |
| M0 | IN PROGRESS | T-001..T-007 implemented; exit blocked on server jar + owner baseline |
| M1 | COMPLETE | T-101..T-104 complete (metrics + runner core + bench harness + harness self-tests) |
| M2 | IN PROGRESS | T-201 complete (device, window, frame loop) |
| M3 / VS-1 | NOT STARTED | — |
| M4–M8 | NOT STARTED | — |

## Performance gate history

| Date | Commit | Scene | Median FPS | p99 / p99.9 (ms) | Result | Run ID |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-11-12 | — | BENCH-00 | ~2,480 | — | BLOCKED (Mailbox, no Immediate on Wayland) | ad hoc |

## Dependency ledger

| Crate | Version | Purpose | License | Unsafe/native boundary | Admitted by |
| --- | --- | --- | --- | --- | --- |
| clap | 4.6.7 | xtask CLI parsing | MIT OR Apache-2.0 | none | T-002 |
| serde | 1.0.229 | doctor report serialization | MIT OR Apache-2.0 | none | T-003 |
| serde_json | 1.0.154 | doctor JSON output | MIT OR Apache-2.0 | none | T-003 |
| wgpu | 30.0.1 | GPU adapter enumeration | MIT OR Apache-2.0 | native GPU bindings (unsafe in deps) | T-003 |
| pollster | 1.0.1 | block on wgpu async adapter enumeration | MIT OR Apache-2.0 | none | T-003 |
| ra-ap-rustc_lexer | 0.184.0 | token-aware Rust comment detection | MIT OR Apache-2.0 | none | T-004 |
| sha2 | 0.11.1 | SHA-256 hashing for reference lock | MIT OR Apache-2.0 | none | T-005 |
| toml | 1.2.1 | bench scene manifest parsing | MIT OR Apache-2.0 | none | T-007 |
| alloc_counter | 0.0.4 | counting global allocator (allocation scope measurement) | MIT OR Apache-2.0 | unsafe in dep (GlobalAlloc) | T-101 |
| command-group | 5.0.1 | process-group spawn/kill for the runner | Apache-2.0 OR MIT | unsafe in dep (nix) | T-102 |
| winit | 0.30.13 | client window and event loop | MIT OR Apache-2.0 | X11/Wayland via dlopen (unsafe in deps) | T-201 |

## Log

Newest entries first. Template:

```
### YYYY-MM-DD · T-NNN title · STATUS
Change:
Files / commit:
Commands and exit codes:
Artifacts (run IDs):
Blockers:
Next smallest action:
```

### 2026-11-12 · T-201 Device, window and frame loop · IMPLEMENTED
Change: Created `sunset-render` (library) and `sunset-client` (binary). sunset-render owns the wgpu instance/adapter/device/surface/queue, present-mode selection (Immediate → Mailbox → Fifo via a tested pure function), surface configure/resize, a one-submit-per-frame clear pass, and the effective present-mode label. sunset-client creates a winit window, runs a continuous-redraw event loop (ControlFlow::Poll + request_redraw, PERF-A14), records frame intervals into a preallocated IntervalRing, handles resize and Outdated/Suboptimal/Lost reconfigure (device-loss recovery), and emits metrics JSON (schema 1: frames/elapsed_s/present_mode/intervals_ms) when launched with a scene manifest path. Validation on in debug / off in release via InstanceFlags (PERF-A11); desired_maximum_frame_latency = 2 (2-slot frame ring, PERF-A04).
Files / commit: crates/sunset-render/*, crates/sunset-client/*, bench/scenes/bench-00.toml, Cargo.toml (members), Cargo.lock (winit 0.30.13 + wayland/x11 stack) (uncommitted).
Commands and exit codes:
  cargo build -p sunset-render → 0
  cargo build -p sunset-client → 0
  cargo test -p sunset-render → 0 (4 present-mode tests)
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask check --non-interactive → 0
  target/debug/sunset-client bench/scenes/bench-00.toml 0 → 0 (emitted metrics JSON)
Artifacts (run IDs): BENCH-00 ad hoc run — ~2,480 FPS (12,480 frames / 5 s), present mode Mailbox.
Blockers: BENCH-00 > 20,000 FPS gate not met on this machine because the Wayland/XWayland compositor offers no Immediate present mode (Mailbox caps throughput); per 09-Benchmarks.md this needs a native X11 session or a tearing-capable Wayland compositor. PERF-A01 "0 allocations/frame" not yet verified: wgpu's per-frame command-encoder allocation is a known open item for the M2 gate (T-212).
Next smallest action: T-202 GPU timing ring.

### 2026-11-11 · T-103 Bench harness · IMPLEMENTED
Change: Added `cargo xtask bench --suite <id> [--gate] [--program <path>]`. Reads bench/scenes/<id>.toml (scene name + timing runs/warmup/sampling), launches the client as `<program> <manifest-path> <run-index>`, captures stdout/stderr (new runner::run_captured with piped stdio + process-group timeout), parses the metrics JSON (schema 1: frames/elapsed_s/present_mode/intervals_ms), computes throughput = frames/elapsed and p99/p99.9 via sunset_metrics::percentile, applies PERF-T1/PERF-T2 from [09](09-Benchmarks.md), flags Fifo present mode, and flags run-to-run CV > 3% as INCONCLUSIVE. Writes runs/<run-id>/bench.json + bench.md. Exit 0 pass / 1 fail or inconclusive / 2 blocked / 3 crash / 4 timeout.
Files / commit: xtask/src/bench.rs (new), xtask/src/runner.rs (run_captured), xtask/src/main.rs (bench subcommand), xtask/Cargo.toml + Cargo.lock (sunset-metrics path dep) (uncommitted).
Commands and exit codes:
  cargo test -p xtask → 0 (25 tests, incl. 6 gate unit tests + 3 end-to-end fake clients: 2900→fail, 3100→pass, noisy→inconclusive)
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask check --non-interactive → 0
  cargo xtask bench --suite bench-01 --gate --non-interactive → 2 (BLOCKED: target/release/sunset-client not built; expected until T-201)
Artifacts (run IDs): bench BLOCKED report written under runs/ (gitignored).
Blockers: none for M1 code. Real BENCH-01 gated run still blocked on the client binary (T-201) and the bench-01 save + owner baseline (owner actions).
Next smallest action: T-104 harness self-tests.

### 2026-11-11 · T-104 Harness self-tests · IMPLEMENTED
Change: Added `cargo xtask test --suite harness`. Runs four injected-fault probes and verifies each is detected for the intended reason: allocation-in-frame-scope (counting allocator via sunset_metrics::count_alloc), 2ms-stall (interval ring max > 0.50 ms budget), timed-out-child (runner RunOutcome::TimedOut), changed-hash (SHA-256 mismatch via lock::hex_sha256). Also hardened the runner against transient process errors: spawn_group retries WouldBlock/Interrupted/ExecutableFileBusy, and both run_child/run_captured retry Interrupted (EINTR) from waitpid; the bench report now distinguishes crashed vs io-error. This fixes an intermittent ETXTBSY failure in the bench fake-client tests under parallel test load.
Files / commit: xtask/src/test.rs (new), xtask/src/main.rs (test subcommand), xtask/src/runner.rs (spawn_group + EINTR/ETXTBSY retry), xtask/src/lock.rs (hex_sha256 pub(crate)), xtask/src/bench.rs (crashed/io-error split) (uncommitted).
Commands and exit codes:
  cargo test -p xtask → 0 (29 tests, incl. 4 harness probe unit tests)
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask test --suite harness --non-interactive → 0 (4/4 faults detected)
  cargo xtask test --suite nope --non-interactive → 2 (unknown suite)
  cargo xtask check --non-interactive → 0
Artifacts (run IDs): none.
Blockers: none for M1 code. M1 code path is now complete (T-101..T-104); M1 exit still blocked on the M2 client binary + bench-01 save + owner baseline.
Next smallest action: T-201 device, window and frame loop (M2).

### 2026-11-11 · T-102 Runner core · IMPLEMENTED
Change: Added the xtask runner core. run_child spawns a child in a process group (command-group), waits with a timeout, kills the whole group on timeout, and classifies outcomes. Exit codes: 0 pass, 1 assertion, 2 missing prerequisite, 3 crash, 4 timeout. Writes runs/<run-id>/report.json + report.md; `cargo xtask run --id --timeout-ms --program [--arg...]` records a run and `cargo xtask report --run <id>` prints it.
Files / commit: xtask/src/runner.rs (new), xtask/src/main.rs (run + report subcommands), xtask/Cargo.toml + Cargo.lock (command-group) (uncommitted).
Commands and exit codes:
  cargo test -p xtask → 0 (16 tests, incl. 5 runner fault tests: hanging→4, crashing→3, missing→2, no-orphan, exit-code mapping)
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask check --non-interactive → 0
  cargo xtask run --id demo --timeout-ms 5000 --program true → 0; cargo xtask report --run demo → 0
Artifacts (run IDs): runs/demo/report.json + report.md.
Blockers: none. JUnit output deferred to T-104.
Next smallest action: T-103 bench harness.

### 2026-11-11 · T-101 sunset-metrics crate · IMPLEMENTED
Change: Created the sunset-metrics crate (safe Rust, no unsafe). IntervalRing (fixed-capacity, no allocation after construction), exact nearest-rank percentile on a sorted copy, frames-per-second throughput, per-frame counters (draws/passes/submits/resource creations/uploads), and the counting global allocator via the audited alloc_counter dependency (the GlobalAlloc unsafe lives in the dependency, per ADR-0001).
Files / commit: crates/sunset-metrics/* (new), Cargo.toml (workspace member), Cargo.lock (uncommitted).
Commands and exit codes:
  cargo test -p sunset-metrics → 0 (6 tests: percentile property test, 1,000,000-interval 0-allocation test, ring/counter/throughput)
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask check --non-interactive → 0 (workspace: fmt + clippy + lint + test)
Artifacts (run IDs): none.
Blockers: none.
Next smallest action: T-102 runner core.

### 2026-11-11 · T-007 Environment and owner baseline · IMPLEMENTED (baseline BLOCKED)
Change: Filled ENVIRONMENT.md "Owner benchmark machine" from the T-003 doctor output. Wrote bench/scenes/bench-01.toml per PRD 4.2 and bench/OWNER-INSTRUCTIONS.md (bench-01 save creation + baseline measurement). PERF-T3 marked BLOCKED (owner action: bench-01 save + ~3,500 FPS settings not provided).
Files / commit: docs/tracking/ENVIRONMENT.md, bench/scenes/bench-01.toml (new), bench/OWNER-INSTRUCTIONS.md (new), xtask/src/main.rs (manifest parse test), xtask/Cargo.toml + Cargo.lock (toml dep) (uncommitted).
Commands and exit codes:
  cargo test --package xtask → 0 (11 tests, incl. bench-01 manifest parse)
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask check --non-interactive → 0
Artifacts (run IDs): none.
Blockers: PERF-T3 (owner baseline) — needs bench-01 save + original ~3,500 FPS settings. T-006 real inventory — needs server jar.
Next smallest action: T-101 sunset-metrics crate (M1, depends on T-004).

### 2026-11-11 · T-006 Inventory via data generator · IMPLEMENTED (real run BLOCKED)
Change: Added `cargo xtask inventory`. Locates Java (system or Prism-bundled) and the server jar, runs the data generator's `--reports` into the private root with a timeout, parses registry report JSON, and writes counts. Parser unit-tested on synthetic registry fixtures. Real run BLOCKED: the 26.3 server jar is not installed.
Files / commit: xtask/src/inventory.rs (new), xtask/src/main.rs, docs/tracking/PARITY.md (target lock recorded) (uncommitted).
Commands and exit codes:
  cargo test --package xtask → 0 (10 tests, incl. 2 inventory parser tests)
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask inventory --non-interactive → 2 ("server jar not found … download the 26.3 server jar")
  cargo xtask check --non-interactive → 0
Artifacts (run IDs): none (real run blocked).
Blockers: 26.3 server jar missing (owner action: download to ../sunset-private/). Data-generator invocation and domain mapping unverified until the server jar is present.
Next smallest action: T-007 environment and owner baseline (blocked on bench-01 save + ~3,500 FPS settings).

### 2026-11-11 · T-005 Reference lock · IMPLEMENTED
Change: Added `cargo xtask lock`. Hashes the local 26.3 client jar and asset index (SHA-256), reads protocol/pack/java from the client jar's version.json, and writes a private target-lock.json (private root) plus a committed target-lock.public.json (identifiers only). Server jar absent → null.
Files / commit: xtask/src/lock.rs (new), xtask/src/main.rs, xtask/Cargo.toml, Cargo.lock, target-lock.public.json (uncommitted).
Commands and exit codes:
  cargo test --package xtask → 0 (8 tests, incl. sha256 known value + version.json parse)
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask lock --non-interactive → 0
  cargo xtask check --non-interactive → 0
Artifacts (run IDs): target-lock.public.json (committed) + ../sunset-private/target-lock.json (private).
Blockers: none. server_jar_sha256 = null (server jar not installed); oracle_adapter_version = NONE.
Next smallest action: T-006 inventory via data generator (needs Java).

### 2026-11-11 · T-004 Source lint and `xtask check` · IMPLEMENTED
Change: Added `cargo xtask lint` and `cargo xtask check`. Lint does token-aware Rust comment detection (ra_ap_rustc_lexer) plus a hand-written WGSL lexer (line + nested block comments), exempts license headers and `#[cfg(test)]` modules/files, and checks `#![forbid(unsafe_code)]` in every crate root. `check` = fmt --check + clippy -D warnings + lint + unit tests.
Files / commit: xtask/src/main.rs, xtask/src/lint.rs (new), xtask/src/check.rs (new), xtask/src/doctor.rs, xtask/Cargo.toml, Cargo.lock (uncommitted).
Commands and exit codes:
  cargo test --package xtask → 0 (6 tests, incl. 4 lint fault fixtures)
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask lint --non-interactive → 0 (clean tree)
  cargo xtask check --non-interactive → 0 (fmt/clippy/lint/test)
  injected comment → cargo xtask lint → 1 ("comment in production Rust source")
Artifacts (run IDs): none.
Blockers: none.
Next smallest action: T-005 reference lock (local 26.3 install path is known).

### 2026-11-11 · T-003 Doctor and result schema v1 · IMPLEMENTED
Change: Implemented `cargo xtask doctor` (replaced the stub). Detects OS/version, CPU model/cores/threads, RAM, Rust toolchain, GPU adapters/backends via wgpu 30 enumeration (async API blocked with pollster), workspace [profile.release] vs PERF-A10, private root (SUNSET_PRIVATE_ROOT or ../sunset-private), and local launcher install (vanilla + Prism/MultiMC native and flatpak). Writes runs/<run-id>/doctor.json (schema 1) and a Markdown summary. Exit 0 pass / 2 blocked / 1 check-fail.
Files / commit: xtask/src/main.rs, xtask/src/doctor.rs, xtask/Cargo.toml, Cargo.lock (uncommitted).
Commands and exit codes:
  cargo test --package xtask → 0 (2 unit tests: exit-code mapping + schema serialization)
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask doctor --non-interactive → 0 (all checks pass)
  HOME=/tmp/empty-home target/debug/xtask doctor --non-interactive → 2 ("No Minecraft installation found…")
Artifacts (run IDs): runs/1794432497/doctor.json + doctor.md.
Blockers: none. Detected: Ubuntu 26.04.1, Ryzen 5 5600X (6c/12t), 16 GB RAM, AMD Radeon RX 6800 XT (Vulkan radv + Gl radeonsi), rustc 1.99.1, Minecraft 26.3 at the Prism flatpak instance path.
Next smallest action: T-004 source lint and `xtask check`.

### 2026-11-11 · T-002 Workspace skeleton and toolchain pin · IMPLEMENTED
Change: Created rust-toolchain.toml (pin 1.99.1, rustfmt+clippy), workspace Cargo.toml ([workspace.lints.rust] unsafe_code = "forbid"; release profile per PERF-A10), .cargo/config.toml (xtask alias + linker), .gitignore, and the xtask crate (clap CLI with a doctor subcommand returning exit 2 "NOT IMPLEMENTED"). Resolved the missing C toolchain: the box has no cc/gcc, so linking goes through a wrapper (~/.local/bin/sunset-ld) around system /usr/bin/ld plus system CRT objects and a libgcc_s.so stub (~/.local/sunset-syslib).
Files / commit: rust-toolchain.toml, Cargo.toml, Cargo.lock, .cargo/config.toml, .gitignore, xtask/Cargo.toml, xtask/src/main.rs (uncommitted).
Commands and exit codes:
  cargo build → 0
  cargo clippy --all-targets -- -D warnings → 0
  cargo xtask doctor --non-interactive → 2 ("NOT IMPLEMENTED")
Artifacts (run IDs): none.
Blockers: none. Toolchain note: no system cc/gcc; future crates with native deps (e.g. wgpu) may need a real C toolchain or zig.
Next smallest action: T-003 doctor and result schema v1.

### 2026-11-11 · T-001 Repository adoption check · IMPLEMENTED
Change: Recorded repository state. The handbook tree is now the active tree: default branch `main` (local and remote) points to handbook commit `107726f`; working tree clean. The prior M4 implementation was discarded and the legacy isometric engine and previous handbook branch remain on the remote as the `archive/legacy-engine` tag and `sunset-handbook-2026-11-09` branch, respectively, per the owner's explicit instruction.
Files / commit: docs/tracking/PROGRESS.md, docs/tracking/BACKLOG.md, HANDOFF.md (tracking updates; uncommitted — T-001 performs no Git writes).
Commands and exit codes (all exit 0):
  git status --short --branch      → clean (## main)
  git branch --show-current        → main
  git symbolic-ref refs/remotes/origin/HEAD → refs/remotes/origin/main
  git rev-parse HEAD               → 107726f5a296cac98379c0955d75027ef0420768
  git branch -a -vv                → main=107726f; origin/sunset-handbook-2026-11-09=bd0cd16
  git tag -l                       → archive/legacy-engine
  git remote -v                    → origin https://github.com/TrissTheBoss/Sunset.git
Artifacts (run IDs): none.
Blockers: none. Later M0 items still need the local 26.3 install (T-005) and owner hardware/baseline (T-007).
Next smallest action: T-002 workspace skeleton and toolchain pin.

### 2026-11-10 · Handbook revision 2 · VERIFIED (documentation only)
Change: PRD rewritten with the 3,000 FPS MUST gate, Fidelity profile, BENCH-01 definition, frame budget, PERF-A requirements, VS-1 slice, MoSCoW priorities and non-goals. Docs 02–17 aligned. Created tracking files, ADR-0001 to ADR-0006, Wiki starters and Cline rules.
Files / commit: documentation only; not committed.
Commands and exit codes: none (no code exists).
Blockers: owner hardware, original ~3,500 FPS settings, local target installation.
Next smallest action: T-001.
