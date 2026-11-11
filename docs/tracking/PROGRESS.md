# Progress

Current milestone: **M0 IN PROGRESS**. Current task: T-002 done (next: [T-003](BACKLOG.md#t-003-doctor-and-result-schema-v1)).

Implementation baseline: fresh rewrite. Results of the previous prototype are not inherited; never carry "M2 complete" from the old implementation into this rewrite.

## Milestone status

| Milestone | Status | Evidence |
| --- | --- | --- |
| M0 | IN PROGRESS | T-001, T-002 complete; workspace skeleton builds clean |
| M1 | NOT STARTED | — |
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
