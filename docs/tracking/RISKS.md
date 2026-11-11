# Risks

Probability/impact: L/M/H. Every risk links to a milestone gate.

| ID | Risk | P | I | Trigger | Mitigation | Gate | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| RISK-01 | 3,000 FPS unreachable because of per-frame API overhead (encoding, submit, present) | M | H | BENCH-00 < 20,000 FPS or BENCH-01S < 4,000 | Measure in T-201 first; indirect draws, one submit, no per-frame creation; ADR with profile before any target change | M2 | OPEN |
| RISK-02 | Present path caps throughput (compositor, Wayland without tearing, Fifo fallback) | M | H | Effective present mode ≠ Immediate/Mailbox | Borderless/exclusive fullscreen, record mode, invalid run on Fifo | M1 | OPEN |
| RISK-03 | Owner hardware or original ~3,500 FPS settings unknown | H | H | ENVIRONMENT still UNKNOWN at M0 exit | T-007 owner instructions; PERF-T3 BLOCKED until recorded | M0 | OPEN |
| RISK-04 | Target 26.3 not verifiable locally | M | H | T-005 BLOCKED | ADR-0003 decision by owner; no silent version switch | M0 | OPEN |
| RISK-05 | wgpu multi-draw-indirect or timestamp support differs per backend/version | M | M | T-208 capability test fails | Looped indirect fallback; ADR-0005 tiers | M2 | OPEN |
| RISK-06 | Real content (models, cutout, water) costs far more than BENCH-01S | M | H | BENCH-01 < 3,000 while BENCH-01S ≥ 4,000 | 33% headroom at M2; calibrate BENCH-01S against BENCH-01 once VS-1 renders | M3 | OPEN |
| RISK-07 | wgpu/winit API churn breaks the build or the agent writes against remembered APIs | H | M | Compile errors after updates | Pin versions; verify APIs in installed sources; upgrades only as dedicated tasks | Continuous | OPEN |
| RISK-08 | Exact Java numeric/RNG semantics drift in Rust | H | H | Differential mismatch | Explicit operations, vectors, no fast-math in simulation | M4 | OPEN |
| RISK-09 | Scope dilution (mods, packs, LOD) delays the pillars | M | H | Non-MUST work started while a gate is red | MoSCoW in PRD; COULD only with green gates | Continuous | OPEN |
| RISK-10 | Legal/branding exposure (EULA, name, authentication app registration) | M | H | Public distribution or account login work | [14](../14-Security-Assets.md); legal assessment before M6/M8 | M6 | OPEN |
| RISK-11 | LLM agent produces plausible but unverified claims or broad rewrites | H | M | PROGRESS without exit codes; diffs outside task scope | BACKLOG scope per task; evidence rules; diff review | Continuous | OPEN |
| RISK-12 | Thermal throttling makes gate runs noisy | M | M | Clock drop or noise > threshold | Warm-up, sensor recording, INCONCLUSIVE instead of pass | M1 | OPEN |
| RISK-13 | Mod sandbox overhead breaks the zero-listener or tick budget | M | H | BENCH-09 over budget | Batched ABI, measured no-op and real modules; ADR-0007 | M6 | OPEN |
| RISK-14 | Rare parity and ordering bugs escape finite tests | H | H | Late differential mismatches | Inventory, trace minimization, oracle fuzzing | Continuous | OPEN |
| RISK-15 | GPU/driver visual nondeterminism makes goldens flaky | M | M | Repeated captures differ | Per-backend calibrated fixtures ([08](../08-Visual-QA.md)) | M1 | OPEN |
| RISK-16 | Save loss or migration incompatibility | M | Critical | Corrupted or refused saves | Work on copies, backups, atomic writes, crash tests | M3–M4 | OPEN |
| RISK-17 | Server anticheat policies differ from vanilla expectations | U | H | Flags on authorized servers | Authorized configuration matrix; treat flags as parity bugs; no universal guarantee | M6 | OPEN |
| RISK-18 | Private GPU runner or account unavailable | M | H | Private lane cannot run | Explicit BLOCKED; public lane stays green | M1 | OPEN |

Probability U = unknown. Do not turn unknown hardware, pack support or model integration into invented facts. Revision 1 risks R-01 to R-12 map to RISK-01, 04, 09 (GLSL, now deferred by ADR-0004), 13, 14, 10, 15, 11, 09, 16, 17 and 18.
