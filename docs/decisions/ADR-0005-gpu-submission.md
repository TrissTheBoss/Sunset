# ADR-0005 Geometry arenas, indirect draws and capability tiers

Status: PROPOSED · Origin: handbook revision 2; accept after T-201/T-208 measurements

Context: at 333 µs per frame, per-section draw calls (thousands per frame at render distance 8) exceed the CPU budget.

Decision (proposed): terrain geometry lives in large sub-allocated arenas. The CPU culls sections and writes a contiguous indirect-draw list per pass. Multi-draw-indirect is used where the pinned wgpu version supports it on the backend; otherwise a looped indirect fallback. A direct-draw path stays as a correctness reference behind a debug toggle. Vertex layout ≤ 16 bytes, fixed in an amendment with T-205.

Capability tiers (to be filled in M2): Tier A, multi-draw-indirect plus timestamps (gated platforms); Tier B, looped indirect; below Tier B unsupported for Fidelity.

Evidence required: BENCH-01S draw counts and timings for both paths; toggle equivalence test.

## Vertex layout (T-205 amendment)

This section fixes the vertex layout placeholder above; it is accepted by T-205 implementation.

The M2 terrain vertex is packed into 8 bytes (64 bits), little-endian, which satisfies the ≤ 16-byte bound and leaves headroom for an M3 vertex carrying entity/material data.

| Field | Bits | Offset | Range | Quantization | Notes |
| --- | --- | --- | --- | --- | --- |
| pos_x | 5 | 0 | 0..16 | integer | section-relative corner coordinate |
| pos_y | 5 | 5 | 0..16 | integer | section-relative corner coordinate |
| pos_z | 5 | 10 | 0..16 | integer | section-relative corner coordinate |
| face | 3 | 15 | 0..5 | integer | Down=0, Up=1, North=2, South=3, West=4, East=5 |
| ao | 2 | 18 | 0..3 | integer | per-corner ambient occlusion |
| light | 4 | 20 | 0..15 | integer | packed block light |
| uv_index | 16 | 24 | 0..65535 | integer | texture-array layer for this face |
| tint_index | 8 | 40 | 0..255 | integer | biome tint palette index (0 = none) |
| reserved | 16 | 48 | 0 | — | reserved for M3; must be written as zero |

Quantization tests (sunset-core `mesh::tests`) verify each field round-trips through its bit width and that reserved bits are zero. The reference mesher writes `ao = 0`, `light = 15`, `tint_index = 0` as neutral placeholders (lighting/tint are populated by T-209); `pos_x/y/z`, `face` and `uv_index` are authoritative. `face` is stored because cube-face winding and UV orientation cannot be recovered from position alone.
