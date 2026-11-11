# Decision index

Lifecycle and template per [15](../15-Tracking-Rules.md). Never delete or rewrite an ADR; supersede it.

| ADR | Title | Status | Origin |
| --- | --- | --- | --- |
| [ADR-0001](ADR-0001-language-and-safety.md) | Safe Rust, original WGSL, no first-party C/C++ or comments | ACCEPTED | Owner mandate |
| [ADR-0002](ADR-0002-performance-target.md) | 3,000 FPS MUST on BENCH-01 in Fidelity native | ACCEPTED | Owner, 10 November 2026 |
| [ADR-0003](ADR-0003-reference-target.md) | Target Java Edition 26.3 | PROPOSED | Old README; awaiting M0 verification |
| [ADR-0004](ADR-0004-shader-pack-scope.md) | Iris/OptiFine compatibility is COULD after M6 | ACCEPTED | Owner, 10 November 2026 |
| [ADR-0005](ADR-0005-gpu-submission.md) | Geometry arenas, indirect draws, capability tiers | PROPOSED | Handbook revision 2 |
| [ADR-0006](ADR-0006-thread-model.md) | Thread model and lock-free snapshot handoff | PROPOSED | Handbook revision 2 |
| ADR-0007 | Wasm module runtime and trusted SDK boundary | RESERVED | Requires M6 safety/performance evidence; create from TEMPLATE.md when the investigation starts |

Revision 1 used a different numbering (reference manifest, Wasm runtime, GLSL strategy, owner workload as ADR-0002 to ADR-0005). Only ADR-0001 had a record; the reserved topics now map to ADR-0003, ADR-0007, ADR-0004 and ADR-0002 respectively. IDs reserve decisions, not fabricated approvals.

New ADRs start from [TEMPLATE.md](TEMPLATE.md).
