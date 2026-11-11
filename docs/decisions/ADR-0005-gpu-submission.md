# ADR-0005 Geometry arenas, indirect draws and capability tiers

Status: PROPOSED · Origin: handbook revision 2; accept after T-201/T-208 measurements

Context: at 333 µs per frame, per-section draw calls (thousands per frame at render distance 8) exceed the CPU budget.

Decision (proposed): terrain geometry lives in large sub-allocated arenas. The CPU culls sections and writes a contiguous indirect-draw list per pass. Multi-draw-indirect is used where the pinned wgpu version supports it on the backend; otherwise a looped indirect fallback. A direct-draw path stays as a correctness reference behind a debug toggle. Vertex layout ≤ 16 bytes, fixed in an amendment with T-205.

Capability tiers (to be filled in M2): Tier A, multi-draw-indirect plus timestamps (gated platforms); Tier B, looped indirect; below Tier B unsupported for Fidelity.

Evidence required: BENCH-01S draw counts and timings for both paths; toggle equivalence test.
