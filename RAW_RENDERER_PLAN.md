# Raw wgpu Renderer — Plan

**Status:** M0–M5 complete; M6–M10 partial (per-deliverable status in §15);
M11–M13 done; M14–M18 planned and unimplemented.
**Date:** 2026-11-08 (amended 2026-11-09 to add core optimizations; amended again to
reconcile completion claims and add correctness-first work).

This plan describes a minimal, custom raw renderer that escapes the fixed
per-frame CPU overhead of Bevy's ECS + render graph. It reuses `sunset-core`
(the authoritative world/simulation) so it is a rendering-adapter swap, not
"mock gameplay" (PRD 11.1). This amendment adds correctness-preserving renderer
optimizations (incremental meshing, compact geometry, visibility, transparency,
resource reuse), using Horizon's client renderer as a technical reference —
adapted to Sunset's data and measured bottlenecks, not copied wholesale.

---

## 1. Objective

Reach high, stable FPS on the `building-yard` fixture (AMD Radeon RX 6800 XT,
Vulkan) while keeping the same world, simulation, player controller, and target
textures. This is now the sole renderer: the Bevy adapter was removed and
`sunset-raw` was renamed to `sunset`.

Per PRD and CORE_PHILOSOPHY: **vanilla parity first, performance second.** An
optimization must not alter required behavior. The 3000 FPS figure is a
**target to measure, not a promised result**; correctness is preserved when an
optimization fails to improve the intended workload.

## 2. Context and the measured bottleneck

Measured (Windows 11, Ryzen 5 5600X, RX 6800 XT, Vulkan):

| State | FPS | Notes |
| --- | --- | --- |
| Original dev build (`opt-level=0` everywhere) | ~90 | CPU-bound, unoptimized Bevy pipeline |
| Optimized deps (`opt-level=3` for Bevy/wgpu) | ~350 | |
| + greedy meshing + MSAA off + combined mesh | ~460 | mesh dropped 40k → **448 triangles** |
| **M0 raw renderer (empty clear-color loop)** | **~2400** | no Bevy, `PresentMode::Immediate` |

The mesh is 448 triangles and the GPU is essentially idle, yet the Bevy adapter
stuck at ~460 FPS (~2.2 ms/frame) because of **Bevy's fixed per-frame CPU cost**
(ECS scheduling, render graph, extraction, command encoding). M0 proved the raw
loop removes that cost (~2400 FPS). Remaining M0→target gap is present/swapchain
overhead plus the dev (unoptimized) build.

## 3. Scope and non-goals

**In scope (baseline renderer, internal only — no user-facing modes/presets)**

- A raw `winit` + `wgpu` loop (no Bevy ECS/render graph/materials).
- Opaque + transparent passes, correct depth behaviour.
- Reuse `sunset-core` world/simulation/player/physics unchanged; fixed 20 Hz tick
  and render interpolation independent of render throughput.
- Greedy meshing (merged rectangles), atlas loader, and all supported block
  shapes/orientations/textures/transparency preserved.
- First-person camera, input, block targeting, break/place.
- Core optimizations (see sections 9–13): incremental meshing, compact geometry,
  GPU arenas, conservative visibility, transparency handling, resource reuse.

**Out of scope (deferred / excluded)**

- LOD and distant-terrain work.
- Compute-generated visibility/draw-list "GPU culling" (the optional mode); a
  separate experimental visibility-buffer path.
- Extra entity/particle/leaf occlusion modes; block-entity model caches;
  shader-pack shadow caching.
- Upscaling, extra AA modes, frame generation, HDR, and other visual extras.
- Text rendering / full HUD parity (glyph atlas/egui) — deferred.
- Networking, worldgen, native save-data probes.
- Texture downsampling or other quality reductions as a "perf win".

## 4. Architecture and reconciliation

`sunset-core` stays the authoritative layer; `sunset` (formerly `sunset-raw`)
is the raw renderer. The Bevy adapter has been removed.

```
sunset-core (world/simulation/physics)   <-- authoritative, unchanged
      ^
sunset (raw wgpu renderer)               <-- sole renderer (Bevy adapter removed)
```

**Reconciliation — combined world mesh is not permanent.** The former Bevy
`renderer.rs` built ONE combined world mesh and rebuilt/uploaded it entirely on
any edit. That was a benchmark shortcut and must not become the renderer's
model. The renderer derives **per-chunk (or per-section) render meshes** from
the authoritative chunks and rebuilds only changed regions (section 9).

Per-frame work of the raw renderer (baseline):

1. Poll `winit` events (keyboard/mouse).
2. Step the simulation if a fixed tick elapsed (20 Hz).
3. Compute the view-projection matrix.
4. Encode one render pass, submit, present.

## 5. Dependencies

Dependencies (the `wgpu`/`winit` versions originate from the former Bevy 0.20
dependency set):

- `wgpu = "30.0.1"` (default features: dx12 + vulkan + metal)
- `winit = "0.30.13"`
- `image` (atlas loader), `glam` (math), `bytemuck` (`Pod` types)
- `sunset-core` (path dependency)
- `pollster` (block on adapter/device creation; already in the lockfile)

## 6. GPU pipeline (baseline vertex format)

Current `MeshData` stores position(f32x3) + normal(f32x3, unused) + color(f32x4)
+ uv(f32x2) = 48 B/vertex. The baseline raw format drops the unused normal:

| Attribute | Type | Bytes |
| --- | --- | --- |
| position | float32x3 | 12 |
| uv | float32x2 | 8 |
| color | float32x4 (face shade) | 16 |

36 B/vertex, interleaved. Compact storage (section 10) shrinks this further.

Uniform: one 4×4 view-projection matrix. Textures: the packed atlas,
`Rgba8UnormSrgb`, nearest sampling. Shader is unlit `textureSample * color`.

Pipeline state: triangle list, cull back; opaque writes depth; transparent tests
depth without writing, alpha-blends `ONE, ONE_MINUS_SRC_ALPHA`; MSAA off
(`sample_count = 1`).

## 7. Render passes

Baseline (current): three physical passes — opaque (clear + write depth),
transparent (load + blend), outline (load). All three use the same colour/depth
attachments and have no intervening consumer requiring a pass boundary.

**M17 — single physical pass.** Fold opaque + transparent + outline into one
`wgpu::RenderPass`:

1. Clear colour and depth once.
2. Draw opaque geometry.
3. Switch pipeline, draw transparency in the required order.
4. Switch pipeline, draw the outline.
5. Store the presented colour.

Preserve blending, depth compare/write behaviour, and outline semantics.
Discard depth only after its final consumer. If the planned visibility work
(§11) introduces a depth read or compute dependency, split at that actual
dependency and preserve the required depth data. Do not assume fewer physical
passes remove necessary pipeline switches or guarantee a GPU speedup — measure.

**Acceptance:** identical rendering; compare CPU encoding time, GPU time, and
total frame time. Reference: `wgpu::StoreOp` (v30.0.1) for store/discard
placement.

The baseline may be one draw call per pass; section 11 relaxes this so
incremental meshes and visibility are not blocked by a "two calls only" rule.

## 8. Reused vs. rewritten

- **Reused unchanged** (`sunset-core`): `World`, `Chunk`, `BlockState`,
  `block_def`/`occludes`/`is_transparent`, `physics::{raycast, move_entity,
  Aabb}`, `simulation`, `player`, `intent`, `fixture`, `save`.
- **Adapted (copy + decouple from Bevy)**: the greedy mesher
  (`greedy_face` + `FACE_NORMALS`/`FACE_SHADE`/`FACE_UVS`/`FACE_AXES`) and the
  atlas loader (`mcassets::load_assets`, already Bevy-independent). These become
  the raw renderer's mesher/assets, emitting raw buffers instead of a Bevy
  `Mesh`. Consolidation into a shared crate happens only if the raw path is kept.
- **New**: window/event loop, wgpu state, camera math, input, and (later) the
  incremental/visibility/storage systems in sections 9–13.

## 9. Incremental mesh construction and scheduling

### 9.0 Correctness prerequisites (M11 — blocking)

Fix two source-verified defects before any meshing or culling change that alters
face visibility or enables backface culling.

**Source evidence (audited at `b038bf6`, re-verified against HEAD):** `mesh::face_visible` decides a face by
whole-cell neighbour visibility only (`neighbor == AIR`, `is_transparent`,
`occludes`) with no surface/direction awareness. `build_chunk_meshes` calls it
for slab faces with the neighbour in the normal direction. A `SlabBottom`'s top
surface (at y+0.5) is therefore culled when the cell above holds a full cube
(whose bottom is at y+1.0); the same applies to a `SlabTop`'s bottom surface.
`mesh::face_corners` gives the `+Y` face the same corner order as `-Y`, so its
winding normal points `-Y` (inward). `main.rs` sets `cull_mode: None` with a
comment admitting the winding is "not consistently front-facing".

**Work it unblocks:** surface-aware occlusion and correct outward winding are
prerequisites for the directional/backface culling in §11 and the word-parallel
visibility fast path in §9.3.

**Requirements:**
- Face visibility must be direction- and surface-aware: a neighbour may suppress
  a face only when its geometry actually covers that surface (compare the face's
  surface plane against the neighbour's `collision_box`, not the whole cell).
  Preserve the current transparency rules (`is_transparent` / `occludes`).
- All six face directions, both triangles, slabs, and greedy rectangles must
  wind outward. Correct the `+Y` corner/index order while preserving texture
  orientation (do not mirror UVs). Two-sided materials keep rendering
  regardless of winding.
- Add independent expected-geometry tests — not just incremental-vs-full-rebuild
  of the same mesher — covering cubes, top/bottom slabs, slab+cube combinations,
  glass, chunk boundaries, merged rectangles, and every face direction,
  asserting the exact face set, winding, and UVs.

**Correctness checks:** the above tests. **Measurements:** none beyond
correctness (this is a gate, not a speed-up).

**Bottleneck addressed:** whole-world rebuild/upload on every edit (the current
combined-mesh shortcut); mesh work stalling the render thread.

**Existing work:** `sunset-core` chunks already carry a monotonic `revision`
bumped by `set_block`; the greedy mesher already exists. Missing: per-region
render meshes, neighbour invalidation, and off-thread scheduling.

**Extends:** M1 (meshes exist), and is a prerequisite for M4 (edits must not
rebuild everything). Milestone **M6**.

**Modules likely affected:** `sunset` new `mesh/` module (region meshes,
dirty map, worker pool); `sunset-core::world` gains nothing (revision already
there) except a documented helper to read neighbour chunks for boundary tests.

Work:

- **Derived per-chunk (or 16×16×16 section) render meshes**, never altering
  authoritative chunk storage. Start with whole-chunk (16×16×64) units; section
  subdivision is a scalability refinement behind real demand.
- **Dirty tracking per region.** A block edit invalidates the affected region
  *and the neighbours whose boundary faces/lighting depend on it.* (The current
  Bevy remesh only re-meshes the edited chunk — a latent boundary-face bug this
  plan must not inherit.)
- **Rebuild/upload only changed regions.** Camera movement never rebuilds.
- **Reject early:** empty regions, fully-hidden faces, and enclosed blocks are
  skipped before model processing, using precomputed block-state facts and bit
  masks that still handle slabs, glass, and oriented blocks correctly.
- **Padded neighbourhood snapshots.** Gather a contiguous, versioned snapshot of
  the region plus the border the mesher actually samples (1 block beyond the
  region for face-visibility). Border size derived from what `greedy_face`
  reads, not a guess.
- **Bounded workers.** `std::thread` workers (bounded count) take immutable
  versioned snapshots; the render thread prioritizes nearby work, coalesces
  repeated invalidations, and rejects stale results by comparing the snapshot
  version against the region's current revision.
- **Scratch reuse.** Reuse worker scratch arrays, face masks, mesh vectors, and
  lookup caches across jobs.
- **Non-blocking collection.** Collect results via a channel without blocking;
  bound queued work, retained snapshots, per-frame completion processing, and
  upload work.
- **Allocation recycling.** Free/recycle large mesh allocations away from the
  render loop so allocator contention doesn't stall frames.

**Correctness invariants / verification:** a boundary edit produces the same
final mesh as a full rebuild; a burst of edits to the same region coalesces to
one final mesh; stale worker output never overwrites newer geometry; no holes
from a dropped job. Unit-test the mesher against full rebuild on real
`sunset-core` data, plus a chunk-boundary edit case.

**Measurements:** mesh build latency (edit→upload), worker CPU, per-frame upload
volume, and frame pacing during an edit burst.

**Fallback:** if worker scheduling regresses, keep the synchronous per-region
rebuild path (correct, just slower).

### 9.1 Border-summary invalidation (M14)

**Source evidence:** `State::sync_meshes` pushes the edited chunk plus all four
horizontal neighbours unconditionally, even for interior edits whose border
faces are unchanged.

**Work it eliminates:** four redundant re-meshes per interior edit.

**Requirements:**
- Use reliable edit coordinates (from `BlockAction::{Break, Place}`) when
  available to invalidate only the owning region; otherwise compare a cached,
  exact border summary when a neighbour's `revision` changes.
- A border summary must capture what the neighbouring mesher actually reads:
  shape coverage, transparency, and the material/visibility rules for the
  supported block set — not a single solid/air bit (a slab must not be
  summarised as "solid").
- Distinguish an owner's material change from a change in the occlusion info it
  exposes to neighbours. Use exact summary equality or explicit dependency
  generations; a probabilistic hash alone must not authorise skipping an
  invalidation.
- Coalesce repeated invalidations; preserve load/unload behaviour.

**Worker-result acceptance (corrects §9's stale-rejection rule):** a mesh
depends on its own data, the sampled neighbour borders, the asset/atlas
generation, and the chunk's lifetime/replacement identity. Reject a completed
job when any dependency is stale — owner `revision` alone is insufficient.

**Acceptance tests:** interior edit (no neighbour rebuild), boundary edit
(neighbour rebuild), border material change with unchanged occlusion (no
rebuild), neighbour replacement, and a late worker result whose neighbour
changed while the owner revision was constant.

**Measurements:** mesh jobs avoided, dependency-tracking overhead, and
edit-to-visible latency.

### 9.2 Face descriptions and compact merge keys (M15)

**Source evidence:** `greedy_face` resolves textures per cell via
`face_textures.get(&state).and_then(texture_for_face).and_then(uv.get(tex))`
(string-keyed atlas lookups in the inner loop), and its per-cell mask payload is
`Option<(BlockState, Option<[f32; 4]>)>`.

**Work it eliminates:** repeated texture-name resolution and string-keyed atlas
lookups; the large per-cell mask payload.

**Requirements:**
- Prepare a renderer-owned face-description table indexed by `(block_state,
  face)` after assets resolve: exact texture/fallback choice, atlas rect, face
  shading, UV orientation, render classification (opaque/transparent), and the
  shape facts meshing needs. Rebuild it when its asset dependencies change.
- Replace the per-cell mask payload with a compact integer merge key plus an
  empty/consumed representation. Preserve the existing merge-equivalence rule:
  do not start merging different block states merely because textures match.
- Reuse masks, visited storage (or consumed entries), mesh-output vectors, and
  other scratch across jobs. If consuming entries replaces the visited array,
  preserve traversal and emitted geometry.
- Keep CPU/GPU packing exact (fallback colours, glass alpha, log orientation,
  slab UV cropping, greedy texture repetition).

**Acceptance:** geometry, materials, winding, and UV behaviour match the
corrected reference mesher (M11). **Measurements:** allocations, scratch size,
lookup counts, total rebuild time, emitted geometry.

Reference (design principle only, no new dependency): `block_mesh::MergeVoxel`'s
separate merge-value contract.

### 9.3 Word-parallel full-cube meshing (M16)

**Source evidence:** `greedy_face` scans each cell individually and calls
`world.get_block` for the neighbour per cell.

**Work it eliminates:** per-cell neighbour lookups and classification for
full-cube faces.

**Requirements:**
- Build compact occupancy/classification rows from the padded chunk snapshot. A
  16-cell row plus a one-cell halo on each end fits in a `u32`; adapt to
  Sunset's actual 16×16×64 dimensions and memory order (do not import a fixed
  62³/64³ representation).
- Derive exposed faces with integer AND/NOT/shift and adjacent-row operations,
  then enumerate visible bits and greedily merge.
- Keep material merge keys separate from occupancy classification; keep a
  correct shape-aware path for slabs and other non-mask geometry; transparent
  visibility follows its real rules, not an opaque occupancy model.
- Preserve merged rectangles, winding, UV repetition, material boundaries, and
  transparent classification; do not split existing greedy rectangles into unit
  faces.

**Acceptance:** equivalence with the corrected reference mesher for empty,
solid, checkerboard, mixed-material, glass, slab, and chunk-boundary cases.
**Measurements:** the complete operation (snapshot gathering, mask
construction, merging, output, upload preparation) — an isolated inner-loop
micro-benchmark is insufficient.

Reference (design principle only): Binary Greedy Meshing's separation of
occupancy generation, word-parallel exposure, and material-compatible merging.

## 10. Compact geometry and GPU storage

**Bottleneck addressed:** vertex/index bandwidth and per-vertex memory.

**Existing work:** greedy meshing already collapses faces into rectangles;
`MeshData` is 48 B/vertex with an unused normal. No compact format or arenas.

**Extends:** M1; milestone **M7**.

**Modules:** `sunset` `mesh/vertex.rs` (packed types), `arena.rs`
(suballocator).

Work:

- **Shrink the vertex.** Position precision sufficient for current/future models
  (e.g. i16×3 relative to a region origin, or f32 where needed). Pack color
  (RGBA8), light/material, and UV into their true domains (e.g. u16 atlas UV,
  u16/u32 material id). Omit normals (shading is flat face shade). Define an
  explicit CPU/GPU layout, alignment, and decode contract.
- **Lossless quad records with shader reconstruction** for eligible geometry.
  Horizon's 24-byte record is a reference, not a target. **Sunset already merges
  faces into rectangles — preserve that.** Store one record per merged
  rectangle (position + orientation + width/height + material + UV origin/scale),
  and reconstruct the 4 corners + 6 indices in the vertex shader (or a shared
  index pattern). Keep the explicit-vertex fallback for any geometry the record
  can't express. Do **not** split merged rectangles back into unit faces to fit
  Horizon's encoding. Preserve UV behaviour, face orientation, color, and any
  per-corner data. Use packing only when the full representation is smaller and
  reconstruction meets precision.
- **Shared GPU mesh arenas:** suballocated ranges with reusable capacity and
  free-range coalescing; device-limit-aware pages; safe lifetime for ranges still
  used by submitted frames; incremental range updates instead of whole-world
  replacement; shared index patterns where Sunset's geometry allows; bounded
  compaction/shrinking.

**Correctness invariants / verification:** pack→reconstruct round-trips
byte-exact for positions/UV/color; merged rectangles, slabs (half-height faces
and half-texture UV), oriented logs, glass, and per-face shade all reconstruct
identically; unsupported geometry falls back to explicit vertices without
visual change.

**Measurements:** bytes/vertex, arena fragmentation, upload volume, and whether
the tiny fixture's FPS is preserved (the simple path must not gain bookkeeping).

**Fallback:** explicit 36-byte vertices remain valid for the fixture; compact
records only where they shrink the total and meet precision.

## 11. Visibility and draw submission

**Bottleneck addressed:** drawing geometry the camera cannot see, and per-draw
CPU overhead at scale.

**Existing work:** none (baseline draws everything with backface culling only).
The tiny fixture needs none of this — these are for the dense-world case.

**Extends:** M6 (dense region storage) and M7 (arenas/batching); milestone **M8**.
Ordered sub-work (each keeps a runnable build):

1. **Hierarchical frustum + distance culling.** Store resident regions densely;
   reject columns/groups before individual bounds where that saves work; bounds
   conservatively contain the actual geometry.
2. **Directional face buckets.** Group suitable faces by orientation so entire
   ranges facing away from the camera are omitted before vertex processing;
   irregular faces go to a conservative bucket; buckets arranged to minimize
   surviving draw ranges. This preserves the merged-rectangle output (a merged
   rectangle lies in one bucket, not split).
3. **Batching + indirect drawing.** Batch compatible draws by material, pipeline,
   and arena page; keep front-to-back opaque ordering (page grouping within
   distance bands); native indirect multi-draw where supported with a correct
   direct-draw fallback.
4. **Cave-connectivity culling.** Conservative connectivity between region
   boundaries; traverse reachable regions from the camera; run graph updates off
   the render thread; newly exposed geometry appears promptly and stale results
   never create holes.
5. **Two-phase depth-based terrain occlusion.** Draw established visible opaque
   geometry, test conservative candidate bounds against current-frame depth, then
   draw newly visible candidates; no synchronous CPU readback; previous
   visibility guides scheduling but is never the sole reason new geometry
   disappears; camera movement/edits/new meshes/camera-inside-bounds all handled
   conservatively.

Use internal workload thresholds so a visibility test never costs more than the
geometry it rejects. Validate both the dense case and the tiny fixture.

**Correctness invariants / verification:** nothing visible is culled when
turning/moving/entering enclosed spaces/breaking an occluder; indirect and
direct paths draw identical geometry in identical order; directional buckets
don't change the rendered image vs. a naive draw.

**Measurements:** draw submissions, culled region count, CPU per frame, and FPS
on a many-region workload; confirm the tiny fixture is not slowed.

**Fallback:** direct draws; full draw when culling is uncertain.

### 11.1 Plane masking and temporal test ordering (M18)

Extend the §11 hierarchical frustum rather than adding a second spatial
structure.

**Requirements:**
- Carry a six-bit active-plane mask from parent bounds to child bounds. A parent
  wholly inside a frustum plane lets its children skip that plane's test; a
  parent outside a plane rejects the subtree. Bounds must conservatively contain
  descendants and be updated before relying on inherited masks.
- A zero mask bypasses frustum tests only — never distance, connectivity, or
  occlusion rules.
- Retain the index of the plane that last rejected a region/group; test it first
  against the current frustum. If it no longer rejects, continue with the
  remaining active planes. This is a test-order hint, never permission to reuse
  a stale hidden result.
- Preserve Sunset's current clip-space convention (DirectX, near z=0) and
  conservative boundary handling. Keep fixed overhead low for tiny scenes.

**Acceptance:** validate against a plain conservative six-plane reference under
movement, rapid rotation, projection changes, negative coordinates, boundary
contacts, and changing mesh bounds. **Measurements:** plane-test counts, culling
CPU time, total frame time.

References (design principle only): Sýkora & Jelínek's evaluation; Cesium's
plane-masking implementation.

## 12. Transparency and shader efficiency

**Bottleneck addressed:** repeated translucent sorting and per-fragment cost.

**Existing work:** glass renders in a separate alpha-blended pass.

**Extends:** M3 (transparent pass); milestone **M9**.

Work:

- Keep translucent vertex data stable; update indices when ordering changes
  (sort the index range, not the vertices).
- Cache valid sort results; limit repeated sorting without accepting visibly
  incorrect blending; deterministic ordering for ties (stable by a region/face
  key).
- Correct opaque/translucent depth: opaque writes depth; translucent tests only,
  drawn back-to-front.
- Batch translucent geometry in shared storage while respecting back-to-front
  order; consider a separate arena when it reduces page switching, recording the
  memory tradeoff.
- Pack vertex→fragment values into fewer interpolant locations without changing
  interpolation or flat-shaded integer semantics.
- Separate truly opaque shading from alpha-tested/fading shading where needed to
  preserve early-depth rejection or tile-GPU hidden-surface rejection.
- Reuse material bindings and deduplicate identical textures while preserving
  the atlas's sampling, orientation, color space, and filtering. Keep the
  current asset pipeline unless a measured limitation justifies changing it.

**Correctness invariants / verification:** back-to-front order and equal-depth
ties are deterministic; blending matches the Bevy reference for the same glass
scene; no texture-quality reduction.

**Measurements:** sort time per frame, draw submissions, and a glass-heavy
fixture's FPS.

**Fallback:** full re-sort each frame when the scene is small.

**Current limitation (source evidence):** `Renderer::render` sorts only whole
chunks back-to-front by chunk-centre distance (§M9). This does not order
translucent surfaces within a chunk, nor surfaces across chunks that straddle
the same depth band. It is correct for the tiny fixture (a single glass wall)
but is not a correctness guarantee for arbitrary translucent geometry.

**Refinement (M9 remainder):** when ordering matters, sort actual translucent
faces/surfaces by a per-face depth key computed once, with a deterministic
tie-breaker. Keep chunk-level sorting only where the scene's translucent faces
are provably disjoint in depth.

## 13. Resource reuse and backend efficiency

**Bottleneck addressed:** per-frame allocation, redundant uploads, and
resource churn.

**Existing work:** the baseline creates nothing per frame beyond command buffers.
M10 currently reuses a chunk's GPU vertex/index buffers on edit (capacity
grow-in-place) only; CPU scratch reuse and shared GPU arenas remain in §9/§10.

**Extends:** M7 (arenas) and M8 (submission); milestone **M10** (backend
refinements last).

Work:

- Reusable upload staging buffers with bounded growth and correct reuse after
  GPU completion (fence/`submit` tracking).
- Reuse shaders, layouts, pipelines, and persistent buffers across ordinary
  frames and compatible reloads; update only when the underlying data changes;
  avoid redundant uploads and per-frame resource creation.
- Backend-appropriate submission paths: where multi-draw is emulated
  inefficiently, evaluate a flattened index/draw path that preserves the same
  geometry and ordering.
- Backend-specific opaque/cutout handling where required for efficient tile
  rendering.
- Command-finishing/submission overlap only when profiling demonstrates a
  benefit on Sunset's Windows/Vulkan target. Do not assume Horizon's Linux
  result transfers.
- Retain development validation and correctness checks; do not prescribe blanket
  validation disabling or copy driver blacklists without a reproduced need.
- GPU sprite animation / GPU model posing attach to the baseline milestones that
  introduce those workloads (upload persistent source once; update compact
  animation/transform data). Do not add missing content just to exercise them.

**Correctness invariants / verification:** no use-after-GPU-free (staging reused
before the GPU is done); no stale data drawn; validation errors stay zero.

**Measurements:** per-frame allocations, upload volume, and FPS.

**Fallback:** straightforward per-frame allocation where it is simpler and
measurably no worse.

### 13.1 Frame-loop dependency rules (M13)

**Source evidence:** `State::render` recomputes the camera, frustum, target
raycast, and transparent ordering every frame; `State::sync_meshes` scans every
resident chunk's `revision` every frame (even with no simulation tick or
mutation); the camera uniform is re-uploaded every frame.

**Work it eliminates:** unchanged derived computation, allocations, and uploads
on warm frames.

**World/mesh sync:** avoid scanning every chunk revision on frames where no
simulation step or mutation occurred; start with the smallest reliable mechanism
the current mutation paths support (explicit edits, load, unload, replacement).
Keep worker-result collection independent so completed meshes still process on
frames without a simulation tick. Do not introduce a general event framework.

**Camera/visibility/ordering/targeting:** use separate dependency keys for:
- camera matrices + frustum (interpolated eye, orientation, projection,
  viewport);
- visible-region lists (camera/frustum state + resident mesh/bounds
  generations);
- transparent ordering (sort-key inputs + transparent-geometry membership);
- target raycast (authoritative eye, direction, reach, relevant world changes);
- camera/outline GPU payload (contents + destination-buffer validity).

The authoritative eye (targeting) and the interpolated eye (rendering) stay
distinct; a render frame without a simulation tick can still have a changed
interpolated camera. Build visibility once and reuse it across opaque and
transparent preparation where their rules agree. Reuse vectors; compute each
sort key once with a deterministic tie-breaker. Skip uniform uploads when
contents are unchanged and the destination already holds valid data (preserve
initial uploads and resource-recreation invalidation). Do not use approximate
camera equality or movement thresholds that introduce lag or visible popping.

**Acceptance:** after warm-up, an unchanged scene avoids repeated derived
calculation and allocation, while moving/editing/resizing/loading match a
forced-recomputation reference.

## 14. Input, camera, targeting, HUD

- **Input** — `winit` `KeyboardInput`/`CursorMoved`, `set_cursor_grab`; build the
  same `PlayerIntent` the Bevy adapter builds.
- **Camera** — eye = `player.pos + (0, eye_height, 0)`; view from yaw/pitch;
  standard perspective (same FOV as the Bevy camera). Interpolate between fixed
  ticks as the Bevy adapter does, independent of render rate.
- **Targeting** — `physics::raycast` from the eye along the look direction;
  12-segment wireframe highlight (static line buffer), or deferred.
- **HUD** — console FPS each second for benchmarking; full text overlay deferred.

## 15. Milestones and sequencing

Preserved milestones first, then optimization milestones. Each milestone ends in
a runnable comparison point (build + measure vs. the previous revision and the
Bevy reference).

- **M0 — Window + clear color.** *Done.* ~2400 FPS on the empty loop, proving
  Bevy's fixed overhead is removed.
- **M1 — Render the world (baseline).** Per-chunk render meshes (not the combined
  world mesh), atlas, opaque pass with depth. Verify visual parity with Bevy
  (same 448-triangle scene).
- **M2 — Camera + input.** View-projection, reuse simulation + physics.
- **M3 — Transparent pass.** Glass (depth-test no-write + alpha blend).
- **M4 — Target + edit.** Raycast highlight + break/place via the simulation.
  Initially may rebuild the affected region synchronously; M6 makes it
  incremental.
- **M5 — Benchmark + compare (baseline).** *Done.* Raw (M4) ~1473 FPS avg vs.
  Bevy ~350–440 FPS avg (see `BENCHMARK.md`), recorded under PRD 7.3
  conditions.
- **M6 — Incremental meshing + scheduling.** *Partial.* Per-chunk meshes and
  revision dirty-tracking are implemented; edits re-mesh the edited chunk
  synchronously. *Not done:* border-summary invalidation (edits always rebuild
  all four horizontal neighbours — §9.1), dependency-aware stale-result
  rejection, and off-thread worker scheduling (the §9 fallback is in use). See
  M14.
- **M7 — Compact geometry + GPU arenas.** *Partial.* Vertex packed 52 → 28 bytes
  (u16 fixed-point UV, RGBA8 color, u16 normalized atlas rect), decoded in the
  vertex shader. *Not done:* quad-record reconstruction and shared GPU mesh
  arenas (deferred).
- **M8 — Visibility + draw submission.** *Partial.* Per-chunk frustum culling
  (Gribb-Hartmann planes, conservative AABB test). *Not done:* the hierarchical
  structure, face buckets, batching/indirect, cave connectivity, and depth
  occlusion. See M18.
- **M9 — Transparency + shader efficiency.** *Partial.* Transparent chunks are
  sorted back-to-front by chunk-centre distance. *Not done:* per-surface
  ordering (a chunk's translucent faces are not mutually ordered), which the
  chunk-level sort does not establish — §12.
- **M10 — Resource reuse + backend efficiency.** *Partial.* A chunk's GPU vertex/
  index buffers are reused on edit when they have capacity. *Not done:* CPU
  scratch reuse, shared GPU arenas, reusable upload staging, and single-pass
  rendering. See M13, M16, M17.

Correctness-first milestones, in dependency order (M11–M13 done, M14–M18
unimplemented):

- **M11 — Correctness prerequisites.** *Done.* (§9.0) Surface-aware slab
  occlusion and outward +Y winding, with independent expected-geometry tests
  (cubes, slabs, slab/cube, glass, merged rectangles, all face directions).
- **M12 — Optimized baseline.** *Done.* (§16.1) Release-build baseline with
  deterministic fixtures, repeated runs, and a precise percentile definition
  (see `BENCHMARK.md` §7).
- **M13 — Frame-loop dependency rules.** *Done.* (§13.1) The per-frame chunk
  revision scan is gated on edits; redundant uniform uploads are skipped; the
  transparent sort vector is reused.
- **M14 — Border-summary invalidation.** (§9.1) Rebuild a chunk's four
  horizontal neighbours only when a border summary actually changed; correct the
  worker-result acceptance rule.
- **M15 — Face descriptions + compact merge keys.** (§9.2) Precompute
  per-state+face render facts; replace the per-cell mask payload with an
  integer merge key; reuse scratch.
- **M16 — Word-parallel full-cube meshing.** (§9.3) u32 occupancy/classification
  rows, bitwise exposure, then greedy merge; shape-aware path for slabs.
- **M17 — Single physical render pass.** (§7) Fold opaque + transparent +
  outline into one `wgpu::RenderPass` with pipeline switches.
- **M18 — Plane masking + temporal ordering.** (§11.1) Extend the M8 hierarchy
  with a six-bit plane mask and last-rejecting-plane hint.

**Sequencing rule:** baseline correction (M12) and geometry correctness (M11)
come first — both gate every later "faster" claim. Small frame-loop (M13) and
invalidation (M14) improvements precede deeper meshing (M15, M16); plane masking
(M18) lands only when the M8 hierarchy is built. Always keep the simple
low-overhead baseline runnable; the tiny fixture must not regress as scalability
work lands. No generic engine framework, no unnecessary new crates, no broad
dependency upgrades, no unrelated gameplay changes.

## 16. Verification and acceptance criteria

Follow the repository's existing checks (`cargo build`, `cargo test`,
`cargo clippy -D warnings`, `cargo fmt --check`) and the PRD's measurement
method (fixed warm-up, 60-second samples, at least three repetitions, on
optimized builds under recorded, comparable conditions).

### 16.1 Optimized baseline (M12)

Establish an optimized baseline of the current revision before accepting any
performance claim:

- Release build with Sunset's own crates optimized (full `opt-level = 3`), not
  the dev "optimized-deps" profile used by the historical §2/M5 numbers.
- Record exact revision, toolchain, build flags, backend, GPU/driver,
  resolution, presentation settings, fixture, and camera path.
- Deterministic fixtures and equal warm-up conditions; repeated runs per the
  repository's sustained-measurement requirement.
- Cover stationary rendering, continuous camera movement, a dense visible
  world, transparent geometry, and block-edit bursts.
- Move console formatting/logging and percentile sorting outside the measured
  interval; keep sample collection bounded and allocation-free during
  measurement.
- Separate CPU frame preparation, meshing, upload preparation, command
  encoding, surface acquisition/presentation waits, and GPU execution where
  supported.
- Define percentile and "0.1% low" metrics precisely; do not compare
  differently-defined statistics under one label.

Every optimization has a correctness gate and an end-to-end performance gate;
report results beyond noise without promising a specific FPS. `BENCHMARK.md`
§2/§6 remain historical records (dev-build and sustained-load numbers
respectively); future claims cite §16.1.

Focused correctness coverage:

- **Expected geometry (M11)** — cubes, top/bottom slabs, slab+cube combinations,
  glass, chunk boundaries, merged rectangles, and every face direction produce
  the exact face set, outward winding, and correct UVs (independent
  expected-geometry tests, not incremental-vs-full-rebuild of the same mesher).
- **Packing/reconstruction** and unsupported-geometry fallback (round-trip exact).
- **Merged rectangles, slabs, oriented logs, textures, glass** — visual parity
  with recorded reference output for the same scene.
- **Chunk-boundary edits and neighbour invalidation** — a boundary edit yields
  the same mesh as a full rebuild; an interior edit rebuilds only the owning
  region (M14).
- **Stale worker results, unloading, and resource replacement** — stale output
  (including a late result whose neighbour changed while the owner revision was
  constant) never overwrites newer geometry; no use-after-GPU-free.
- **Visibility** while turning, moving, entering enclosed spaces, and breaking an
  occluder — nothing visible is culled; plane masking matches a plain six-plane
  reference (M18).
- **Transparency ordering and equal-depth cases** — deterministic per-surface
  ordering.
- **Simulation and input outcomes remain independent of render rate** (fixed tick
  and interpolation unchanged).

Preserve the `building-yard` fixture; add narrowly targeted workloads (edit
bursts, many resident regions, occlusion, transparency) only as each
implementation lands, using real `sunset-core` world data.

Measure (optimized builds): CPU and GPU frame cost separately where available,
frame-time percentiles, draw submissions, geometry counts, upload volume, mesh
latency, and memory. Compare each optimization against the previous renderer
revision and the §16.1 baseline under equivalent conditions.

## 17. Success criteria

- Rendering visually matches recorded reference output for the same world +
  textures.
- Movement, targeting, and block edit behave identically (same `sunset-core`).
- Edits do not rebuild/upload the whole world (per-region updates only; an
  interior edit rebuilds only the owning region — M14).
- Completion is claimed per deliverable, not per milestone (see §15 statuses).
- 3000+ FPS on the fixture is a **target to measure**, not a promise; report the
  measured number against the §16.1 baseline and the previous revision.
- A written benchmark record (revision, fixture, config, FPS, percentiles) per
  PRD 7.3 and §16.1.
- Correctness is preserved whenever an optimization fails to improve the
  intended workload.

## 18. Risks and unknowns

1. **wgpu 30 / winit 0.30 API drift** from older tutorials (already resolved for
   M0; re-check for compute/indirect/timestamp queries).
2. **Present overhead** — even `Immediate` has driver/swapchain cost; the ceiling
   may be ~2000–4000 FPS rather than unbounded.
3. **Boundary invalidation** — neighbour-chunk faces must be invalidated; getting
   this wrong reintroduces the latent combined-mesh bug.
4. **Stale worker results** — versioned snapshots must reject late results.
5. **Occlusion holes** — conservative depth/cave culling must never hide new or
   just-revealed geometry.
6. **Compact-record precision** — packed position/UV must round-trip without
   visible seams; merged rectangles complicate a fixed-size record.
7. **Transparency determinism** — ties and sort caching must not change blending.
8. **Tiny-fixture regression** — scalability machinery must not add bookkeeping
   to an almost-empty frame.
9. **Texture format** — atlas `Rgba8UnormSrgb` sampler/shader must match.
10. **Duplication** — mesher/atlas temporarily duplicated until consolidation.
11. **Surface-aware occlusion** — a slab face wrongly culled or kept adds or
    removes geometry (M11).
12. **Winding / backface culling** — enabling backface culling before every face
    winds outward would hide geometry (M11).
13. **Single-pass merging** — folding passes could change depth/blend ordering or
    outline semantics (M17).
14. **Plane-mask drift** — inherited masks must update with bounds; a stale mask
    culls visible geometry (M18).

## 19. Decision log

- **`sunset-raw` became `sunset`** — the Bevy app was removed once the raw path
  was validated, so the raw renderer is now the only renderer.
- **Reuse `sunset-core` unchanged** — the world/simulation is renderer-agnostic.
- **Per-chunk (or section) derived render meshes, not the combined world mesh** —
  the combined mesh was a benchmark shortcut; it must not force whole-world
  rebuilds on edit.
- **Copy the mesher/atlas now, consolidate into `sunset-render-common` only if
  the raw path is kept** — avoid premature cross-crate refactor.
- **Keep the explicit-vertex fallback; add compact quad records only where they
  shrink and meet precision** — preserve the existing merged-rectangle advantage
  instead of fitting Horizon's unit-face encoding.
- **Conservative, workload-thresholded visibility; no synchronous depth readback**
  — correctness over aggressive culling.
- **3000 FPS is a measurement target, not a promise** (PRD/CORE_PHILOSOPHY).
- **Correctness-first ordering** — M11 (geometry) and M12 (baseline) gate every
  speed claim; a milestone is *partial* unless every listed deliverable exists.
- **Surface-aware occlusion and corrected winding are prerequisites**, not
  optional — they gate the directional/backface culling already planned in §11.
- **One physical pass (M17) only where no consumer needs a boundary** — measure,
  do not assume a speed-up.
- **External references are design principles, not dependencies** — `block_mesh`
  (`MergeVoxel`), Binary Greedy Meshing, Sýkora & Jelínek, and Cesium are cited
  for their approach only.

## 20. Amendment summary (2026-11-09)

This amendment reconciles the plan's completion claims against the actual source
(audited at `b038bf6`, re-verified against HEAD) and adds correctness-first
optimization work. What changed and why:

- **Completion claims corrected.** M6–M10 were marked *done*; the source shows
  only one deliverable of each exists (frustum culling, chunk-level transparent
  sort, GPU buffer-capacity reuse, the 28-byte vertex). They are now *partial*,
  and the missing pieces (border summaries, per-surface ordering, arenas,
  off-thread scheduling, single-pass) are re-scoped as explicit milestones.
- **Geometry correctness first.** Two source-verified defects (whole-cell slab
  face visibility; +Y inward winding hidden by `cull_mode: None`) block any
  meshing/culling change that alters face visibility or enables backface
  culling. M11 fixes them and adds independent expected-geometry tests.
- **Baseline correction first.** M12 requires a release-build, deterministic,
  repeated baseline with precise percentile definitions before any performance
  claim, superseding the historical dev-build numbers in `BENCHMARK.md`.
- **Frame-loop and invalidation before deeper meshing.** M13 (dependency-keyed
  derived work) and M14 (border-summary invalidation) are small, and precede the
  larger M15/M16 meshing work.
- **Plane masking integrated into the existing hierarchy**, not a second
  structure (M18, gated on the M8 hierarchy).
- **Stale references fixed.** The architecture diagram and prose no longer assume
  a separate `sunset-raw` application or an active Bevy renderer.
- **Existing requirements preserved, not duplicated.** Worker scheduling, arenas,
  compact geometry, batching, and visibility remain in §9–§13; their status is
  clarified in §15 rather than restated.

The amendment itself made no code changes. M11–M13 were subsequently
implemented and M14–M18 remain unimplemented (see §15 statuses).





