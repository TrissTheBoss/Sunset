# ADR-0004 Iris/OptiFine shader pack compatibility is COULD after M6

Status: ACCEPTED · Origin: owner, 10 November 2026

Context: GLSL pack compatibility requires reproducing OpenGL conventions, uniforms and pack pipeline semantics, which is a large effort that competes with the performance and fidelity pillars.

Decision: REQ-14 is split. REQ-14a native Sunset WGSL packs: SHOULD, M6. REQ-14b Iris/OptiFine-format compatibility: COULD, no work before M6, then a feasibility spike and ADR. The M2 pass structure declares its resources and order in data so the option stays open.

Reversal: owner decision after M6.
