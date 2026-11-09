# Raw wgpu Renderer — Plan

**Status:** M0–M4 implemented and measured; M5+ planned (M5 = benchmark, M6+ = optimizations).
**Date:** 2026-11-08 (amended to add core renderer optimizations).

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
textures. The Bevy adapter remains the reference for correctness; the raw
renderer is the performance path.

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

`sunset-core` stays the authoritative layer; `sunset-raw` is the raw renderer;
`sunset` (Bevy) stays as the comparison path. All three already exist.

```
sunset-core (world/simulation/physics)   <-- authoritative, unchanged
      ^                      ^
sunset (Bevy)           sunset-raw (raw wgpu)
comparison path         target path
```

**Reconciliation — combined world mesh is not permanent.** The current Bevy
`renderer.rs` builds ONE combined world mesh and rebuilds/upload it entirely on
any edit (`build_world_meshes` + `remesh_system`). That was a benchmark
shortcut and must not become the raw renderer's model. The raw renderer derives
**per-chunk (or per-section) render meshes** from the authoritative chunks, and
rebuilds only changed regions (section 9). The combined-mesh shortcut in the
Bevy adapter may stay for comparison, but it is not the design the raw renderer
adopts.

Per-frame work of the raw renderer (baseline):

1. Poll `winit` events (keyboard/mouse).
2. Step the simulation if a fixed tick elapsed (20 Hz).
3. Compute the view-projection matrix.
4. Encode one render pass, submit, present.

## 5. Dependencies

Pin the versions Bevy 0.20 uses (so asset/format work matches):

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

1. Opaque pass — clear, write depth, draw opaque geometry.
2. Transparent pass — same depth (test only), alpha blend, draw glass.

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

**Bottleneck addressed:** whole-world rebuild/upload on every edit (the current
combined-mesh shortcut); mesh work stalling the render thread.

**Existing work:** `sunset-core` chunks already carry a monotonic `revision`
bumped by `set_block`; the greedy mesher already exists. Missing: per-region
render meshes, neighbour invalidation, and off-thread scheduling.

**Extends:** M1 (meshes exist), and is a prerequisite for M4 (edits must not
rebuild everything). Milestone **M6**.

**Modules likely affected:** `sunset-raw` new `mesh/` module (region meshes,
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

## 10. Compact geometry and GPU storage

**Bottleneck addressed:** vertex/index bandwidth and per-vertex memory.

**Existing work:** greedy meshing already collapses faces into rectangles;
`MeshData` is 48 B/vertex with an unused normal. No compact format or arenas.

**Extends:** M1; milestone **M7**.

**Modules:** `sunset-raw` `mesh/vertex.rs` (packed types), `arena.rs`
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

## 13. Resource reuse and backend efficiency

**Bottleneck addressed:** per-frame allocation, redundant uploads, and
resource churn.

**Existing work:** the baseline creates nothing per frame beyond command buffers.

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
- **M5 — Benchmark + compare (baseline).** Record FPS (avg + 0.1% low) vs. the
  previous raw revision and the Bevy adapter under PRD 7.3 conditions.
- **M6 — Incremental meshing + scheduling.** Section 9. Prereq: M1. Unlocks M4
  being truly incremental.
- **M7 — Compact geometry + GPU arenas.** Section 10. Prereq: M1.
- **M8 — Visibility + draw submission.** Section 11. Prereq: M6, M7.
- **M9 — Transparency + shader efficiency.** Section 12. Prereq: M3.
- **M10 — Resource reuse + backend efficiency.** Section 13. Prereq: M7, M8.

**Sequencing rule:** always keep the simple low-overhead baseline runnable; the
tiny fixture must not regress as scalability work lands. Advanced visibility
(M8) stays behind its actual prerequisites (M6 dense storage, M7 arenas), and
backend refinements (M10) come last. No generic engine framework, no unnecessary
new crates, no broad dependency upgrades, no unrelated gameplay changes.

## 16. Verification and acceptance criteria

Follow the repository's existing checks (`cargo build`, `cargo test`,
`cargo clippy -D warnings`, `cargo fmt --check`) and the PRD's measurement
method (fixed warm-up, 60-second samples, at least three repetitions, on
optimized builds under recorded, comparable conditions).

Focused correctness coverage:

- **Packing/reconstruction** and unsupported-geometry fallback (round-trip exact).
- **Merged rectangles, slabs, oriented logs, textures, glass** — visual parity
  with the Bevy reference for the same scene.
- **Chunk-boundary edits and neighbour invalidation** — a boundary edit yields
  the same mesh as a full rebuild.
- **Stale worker results, unloading, and resource replacement** — stale output
  never overwrites newer geometry; no use-after-GPU-free.
- **Visibility** while turning, moving, entering enclosed spaces, and breaking an
  occluder — nothing visible is culled.
- **Transparency ordering and equal-depth cases** — deterministic, matches Bevy.
- **Simulation and input outcomes remain independent of render rate** (fixed tick
  and interpolation unchanged).

Preserve the `building-yard` fixture; add narrowly targeted workloads (edit
bursts, many resident regions, occlusion, transparency) only as each
implementation lands, using real `sunset-core` world data.

Measure (optimized builds): CPU and GPU frame cost separately where available,
frame-time percentiles, draw submissions, geometry counts, upload volume, mesh
latency, and memory. Compare each optimization against the previous raw-renderer
revision and the Bevy reference under equivalent conditions.

## 17. Success criteria

- Rendering visually matches the Bevy adapter for the same world + textures.
- Movement, targeting, and block edit behave identically (same `sunset-core`).
- Edits do not rebuild/upload the whole world (per-region updates only).
- 3000+ FPS on the fixture is a **target to measure**, not a promise; report the
  measured number and the comparison against the previous raw revision and the
  Bevy reference.
- A written benchmark record (revision, fixture, config, FPS, percentiles) per
  PRD 7.3.
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

## 19. Decision log

- **`sunset-raw` as a separate crate** — keeps the Bevy app for A/B comparison.
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





