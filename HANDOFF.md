# Working handoff

Goal: fresh Sunset rewrite, custom Rust/WGSL foundation first, exact target behavior, and 3,000 FPS MUST on BENCH-01 (Fidelity, native, owner machine).

Completed: handbook revision 2 (10 November 2026): PRD with performance contract, aligned docs 02–17, tracking files, ADR-0001 to ADR-0006, backlog M0–M3, Wiki starters, Cline rules. No game implementation in this tree.
Current implementation milestone: M0 NOT STARTED.
Target: Java Edition 26.3 proposed (ADR-0003), not yet artifact-verified.
Hardware: unknown; ~3,500 FPS is an unverified owner report, settings unknown (RISK-03).
Constraints: no first-party C/C++; safe Rust; no existing renderer; local legal assets; no production source comments; preserve recoverable Git history; PERF-A01–A15.
Decisions: docs/decisions/INDEX.md.
Checks: document review only; no game/compiler/GPU/oracle test executed.
Next action: T-001 in docs/tracking/BACKLOG.md, using docs/12-First-Task.md as the kickoff prompt.
Open owner actions: record hardware and original ~3,500 FPS settings; confirm the local 26.3 install; create the bench-01 save (instructions follow in T-007).
