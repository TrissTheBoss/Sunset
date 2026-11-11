# Working handoff

Goal: fresh Sunset rewrite, custom Rust/WGSL foundation first, exact target behavior, and 3,000 FPS MUST on BENCH-01 (Fidelity, native, owner machine).

Completed: handbook revision 2 (10 November 2026): PRD with performance contract, aligned docs 02–17, tracking files, ADR-0001 to ADR-0006, backlog M0–M3, Wiki starters, Cline rules. No game implementation in this tree. T-001 repository adoption check done: handbook force-pushed to `main` as the active tree (owner authorized a one-time discard of the prior M4 implementation and reset of the recoverable-history constraint). T-002 workspace skeleton and toolchain pin done: workspace builds, clippy clean, doctor stub returns 2. T-003 doctor done: full environment/GPU/launcher detection writing doctor.json + Markdown summary. T-004 source lint and `xtask check` done: token-aware comment lint + WGSL lexer + check pipeline.
Current implementation milestone: M0 IN PROGRESS (T-001..T-004 done).
Target: Java Edition 26.3 proposed (ADR-0003), not yet artifact-verified.
Hardware (detected by T-003 doctor): Ubuntu 26.04.1; AMD Ryzen 5 5600X (6 cores / 12 threads); 16 GB RAM; AMD Radeon RX 6800 XT (Vulkan radv + Gl radeonsi). ~3,500 FPS is an unverified owner report, settings unknown (RISK-03).
Toolchain: Rust 1.99.1 pinned; no system cc/gcc, so linking uses ~/.local/bin/sunset-ld (wrapper around system ld + system CRT) and ~/.local/sunset-syslib/libgcc_s.so.
Constraints: no first-party C/C++; safe Rust; no existing renderer; local legal assets; no production source comments; preserve recoverable Git history; PERF-A01–A15.
Decisions: docs/decisions/INDEX.md.
Checks: cargo build → 0; cargo clippy --all-targets -- -D warnings → 0; cargo test --package xtask → 0 (6 tests); cargo xtask doctor --non-interactive → 0 (real detection, blocked path verified → 2); cargo xtask check --non-interactive → 0. No game/oracle test executed yet.
Next action: T-005 in docs/tracking/BACKLOG.md.
Open owner actions: create the bench-01 save (instructions in T-007); record the original ~3,500 FPS settings. Minecraft 26.3 install located at /home/triss/.var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances/26.3/minecraft (Prism Launcher).
