# Custom renderer, LOD, upscaling and settings

## Pipeline

Start with capabilities, a correct CPU reference mesher and explicit passes. Add a small pass graph only when packs/effects need it. Immutable scene → visibility → opaque/cutout → entities/block entities → sky/effects → transparency → reconstruction/post → correct hand/HUD/UI composition. Fidelity follows actual target ordering; Enhanced declares pass resources/depth conventions.

Cache pipelines/bind groups; batch compatible geometry. Texture arrays or padded atlases must support arbitrary pack dimensions, animation, UV rotation, model inheritance, tint/cutout/translucency. Non-cube models remain non-cubes. Fidelity preserves lightmaps/AO/fog/sampling. CPU section frusta write a contiguous indirect-draw list over sub-allocated geometry arenas from the first renderer (PERF-A05): per-section direct draws produce thousands of calls per frame and cannot fit the 250 µs CPU budget. A plain direct-draw path is kept only as a correctness reference and capability fallback, and toggle-equivalence tests compare both. GPU culling/compaction (OPT-15) is adopted only after measurement. Handle shader failure, resize/minimize, device loss and reload.

## Hot-path rules for 3,000 FPS

- All pipelines, bind group layouts and shader modules are created at load or reload, never during a frame (PERF-A06). Use a pipeline cache where the backend supports it.
- Bind groups for terrain: one global (camera/fog/time uniform in a ring-indexed dynamic-offset buffer), one material (texture array plus sampler), one geometry arena. Switch pipelines at most once per pass.
- Vertex format: packed, at most 16 bytes per vertex in M2 (section-relative position, UV index, packed light/AO, tint index). The exact layout is fixed by ADR in M2 with quantization tests.
- Geometry arenas: large fixed-size buffers with a free-list sub-allocator; fragmentation and compaction are measured in BENCH-06.
- Present-mode selection order: Immediate → Mailbox → Fifo. Record the effective mode; a gated benchmark with Fifo is invalid.
- HUD/UI: one atlas, one batched vertex stream, one draw per material class.
- No readback on the frame path. Timestamp and statistics queries resolve into a ring and are read 2 frames later.

## Greedy meshing

Only merge coplanar exposed faces with equivalent material/normal/layer/UV/tint/light/AO interpolation. Matching corner values is not sufficient if the large rectangle interpolates differently. Split when equivalence cannot be proved. Repeat textures without stretching or mip/atlas bleeding. Non-cubic/animated/translucent content may use dedicated paths.

Keep a simple culled-face oracle and compare surface/raster output. Temporary edit meshes and refined greedy meshes both remain correct. Neighbor revisions prevent seams. Leaves/cutout models are not automatically opaque occluders.

## LOD

Use camera-centered, incrementally updated bands of voxel-derived coarse meshes. Preserve overhangs/caves/bridges with multilayer or coarse voxel surfaces; a single heightfield cannot represent all geometry. Clipmaps inspire bounded updates, not copied code.

Near range stays exact. LOD is an Enhanced feature and must keep Enhanced ≥ Fidelity throughput (PERF-T5). Experimental distances: exact through 128 blocks, coarse bands around 256/512/1024, adjusted to view distance/screen-space error/hardware. These are proposals. Hysteresis prevents thrashing. Verify stitched transitions from all directions; skirts may create false walls. Bounded dithering/dual rendering needs visual/cost tests. Water stays separate.

Cache identity includes world/server/dimension, revisions, target/generator, packs/materials and algorithm version. Edits invalidate ancestors; eviction has byte caps. On vanilla servers use only received terrain or explicitly enabled history of previously received terrain. Never guess hidden/unreceived terrain or share caches across servers. LOD never alters simulation.

## Upscaling

Ship Off/Auto/fixed scales. Enhanced defaults Auto, which may choose 100% native if reconstruction is slower or quality fails. Fidelity forces native. Start with modest custom WGSL spatial reconstruction against nearest/bilinear/native references. Temporal reconstruction is later: motion vectors, disocclusion rejection, reactive masks, cut/reload history reset.

Dynamic scale uses smoothed GPU time, bounded changes/hysteresis; CPU bottlenecks do not justify lower resolution. HUD/text remains native. Report both dimensions. Frame generation never counts toward the 3,000 FPS gate or the 4,000 FPS stretch. Vendor SDKs are not assumed.

## Settings

Vanilla-style navigation and local UI/font assets. Categories: Video, Performance, LOD, Upscaling, Shader Packs, Resource Packs, Controls, Audio, Accessibility, Language, Mods, Privacy, Diagnostics. Include display/fullscreen/VSync/cap/render and simulation distances/profile/brightness. Safe Auto worker/cache controls; unsupported capabilities are explained, not falsely enabled.

Each setting has stable key/type/range/default/persistence version/reload semantics/test ID. Per-category reset. Display changes roll back after 15 seconds without confirmation. Test keyboard/mouse/focus/scaling/long translations/RTL where applicable/narration. Failed pack reload keeps the prior state.

Foundation scenes cover authored opaque/cutout/translucent geometry, arbitrary models, moving chests/entities, particles and fonts. Pass seams/precision/resize/reload/device-loss tests before mass content.
