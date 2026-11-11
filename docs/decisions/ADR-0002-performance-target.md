# ADR-0002 3,000 FPS MUST on BENCH-01 in Fidelity native

Status: ACCEPTED · Origin: owner, 10 November 2026

Context: the handbook previously treated 4,000 FPS as a stretch, so performance was not a gate. The owner declared high performance with 3,000 FPS a must-have. The owner reported ~3,500 FPS with an existing client; hardware and settings are not yet recorded.

Decision: PERF-T1/PERF-T2 in [PRD](../01-PRD.md) section 4 are MUST gates on the BENCH-01 definition in section 4.2, measured in Vanilla Fidelity at native resolution on the owner machine. 4,000 FPS remains a STRETCH. Gates per milestone: M2 BENCH-01S ≥ 4,000, M3 BENCH-01 ≥ 3,000, from M4 BENCH-01Q on every merge.

Why: a must-have without a fixed scene cannot pass or fail; the Fidelity requirement prevents meeting the number through upscaling or LOD; the M2 headroom catches architectural shortfalls before content exists.

Costs: architecture constraints PERF-A01–A15; a trusted GPU runner is needed for per-merge gates; some features wait while the gate is red.

Pending amendment: when M0 records the owner's original ~3,500 FPS settings and they differ from section 4.2, this ADR is amended (not superseded) before the M2 gate, keeping both definitions.

Reversal: only by the owner, with measured evidence of the bottleneck.
