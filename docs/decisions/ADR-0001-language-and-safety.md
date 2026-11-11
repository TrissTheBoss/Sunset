# ADR-0001 Custom safe Rust/WGSL foundation and profiles

Date: 9 November 2026, revised 10 November 2026.
Status: ACCEPTED for user-mandated constraints; implementation defaults remain subject to measured revision.

Context: full Java Edition behavior, very high FPS, modding, no first-party C/C++, a custom renderer and local resources are required. LOD and upscaling change pixels.

Decision: an original application in safe Rust (`#![forbid(unsafe_code)]` in every first-party crate) with original WGSL rendering. A GPU abstraction (wgpu) is allowed; reuse of a complete renderer is excluded. Dependencies may contain audited unsafe/native code. First-party production Rust/WGSL contains no comments; design explanation lives in Markdown; license notices are preserved; tests may contain explanatory comments. Vanilla Fidelity is separated from Enhanced. Measurement, extension and ownership infrastructure comes before mass content, using representative synthetic fixtures. Behavioral rules are preserved in both profiles.

Alternatives: an existing game renderer, rejected by user requirement; GPU-authoritative gameplay, rejected for determinism and compatibility risk; strict visual equality with active lossy LOD/upscaling, rejected as contradictory.

Consequences: significant original engineering; multiple comparison profiles; GLSL packs require extra work ([ADR-0004](ADR-0004-shader-pack-scope.md)); dependency/driver internals still need an audit; performance techniques that would need first-party unsafe must come from audited dependencies or safe wgpu APIs; agents need strong Markdown contracts because code carries no comments. The performance target is set by [ADR-0002](ADR-0002-performance-target.md) (3,000 FPS MUST), which replaces the earlier "4,000 FPS stretch only" statement of this record.

Evidence: `cargo xtask lint` (T-004). Reversal: implementation technologies/defaults require a new ADR and relevant tests; user constraints require a user decision.
