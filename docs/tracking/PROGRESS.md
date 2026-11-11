# Progress

Current milestone: **M1 IN PROGRESS** (M0 code complete, exit blocked on owner actions). Current task: T-101 done (next: [T-102](BACKLOG.md#t-102-runner-core)).

Implementation baseline: fresh rewrite. Results of the previous prototype are not inherited; never carry "M2 complete" from the old implementation into this rewrite.

## Milestone status

| Milestone | Status | Evidence |
| --- | --- | --- |
| M0 | IN PROGRESS | T-001..T-007 implemented; exit blocked on server jar + owner baseline |
| M1 | IN PROGRESS | T-101 sunset-metrics crate complete |
| M2 | NOT STARTED | — |
| M3 / VS-1 | NOT STARTED | — |
| M4–M8 | NOT STARTED | — |

## Performance gate history

| Date | Commit | Scene | Median FPS | p99 / p99.9 (ms) | Result | Run ID |
| --- | --- | --- | --- | --- | --- | --- |
| — | — | — | — | — | No measurements yet | — |

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
