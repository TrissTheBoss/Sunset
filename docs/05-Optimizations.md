# Optimization catalog

Admit an optimization only with reproducible workload, before/after timings/memory, correctness evidence and rollback path. P0 = foundation; P1 = profiling-driven; P2 = research. Comprehensive culling means appropriate coverage of every drawable class, not every algorithm on every object.

| ID / priority | Technique | Benefit / required safeguard |
| --- | --- | --- |
| OPT-01 P0 | Hidden block-face removal | Fewer primitives; shape-aware partial/translucent neighbor tests |
| OPT-02 P0 | Section/chunk frusta | Skip offscreen geometry; conservative bounds |
| OPT-03 P0 | Entity/chest/block-entity/particle frusta | Animated bounds, labels, outlines and shadows tested |
| OPT-04 P0 | Backface culling | Two-sided materials exempt |
| OPT-05 P0 | Greedy meshing | Material/light/AO/UV equivalence and edit latency |
| OPT-06 P0 | Dirty-section rebuild/cached meshes | Neighbor/light invalidation and stale job rejection |
| OPT-07 P0 | Batching/instancing | Preserve transparency order and instance state |
| OPT-08 P0 | Packed vertices/indexing | Quantization, alignment and large-coordinate tests |
| OPT-09 P0 | Reused buffers/staging rings | Respect GPU-in-flight lifetimes; bounded memory |
| OPT-10 P0 | Palettes/dense hot arrays | Stable IDs and exact round trips |
| OPT-11 P0 | Job priorities/backpressure | Stable simulation commits and cancellation |
| OPT-12 P0 | Pipeline/bind caches | Asset-epoch invalidation and size limits |
| OPT-13 P1 | Connectivity/cave occlusion | Test doors, narrow openings and partial geometry |
| OPT-14 P1 | Hierarchical-Z occlusion | Conservative depth; rapid turns/teleports/occluder removal |
| OPT-15 P1 | GPU indirect compaction | Feature gates and direct-draw fallback |
| OPT-16 P1 | Front-to-back/optional depth prepass | Overdraw savings must beat extra-pass cost |
| OPT-17 P1 | Distance/screen-size culling | Fidelity follows vanilla visibility; aggressive reduction enhanced-only |
| OPT-18 P1 | Shadow-pass culling | Offscreen casters still affect visible shadows |
| OPT-19 P1 | Incremental LOD bands | Enhanced-only seams/quality/cache correctness |
| OPT-20 P1 | Spatial upscaling/dynamic resolution | GPU-bound benefit and quality gates |
| OPT-21 P1 | SIMD/bitset meshing | Scalar equality and portable CPU fallback |
| OPT-22 P1 | Incremental lighting | Exact result/order against full recomputation |
| OPT-23 P1 | Spatial collision/query indices | Same final candidate decisions and tie order |
| OPT-24 P1 | IO batching/compression caches | Atomic durability and bounded memory |
| OPT-25 P1 | PGO/LTO/codegen tuning | Held-out workloads and reproducible build metadata |
| OPT-26 P1 | Batched mod events | No-hook overhead and deterministic ordering |
| OPT-27 P1 | Mipmap/anisotropy tuning | Fidelity sampling and atlas-border/oblique tests |
| OPT-28 P2 | Mesh clusters/meshlets | Metadata/compute savings measured against section culling |
| OPT-29 P2 | GPU meshing | CPU oracle, halo synchronization and edit latency |
| OPT-30 P2 | Async compute | GPU timeline proves overlap rather than contention |
| OPT-31 P2 | Temporal upscaling | Ghosting, motion vectors, reactive masks and history resets |
| OPT-32 P2 | Advanced/OIT transparency | Enhanced option; preserve Fidelity ordering |
| OPT-33 P2 | Virtual textures/sparse residency | Only if huge packs justify complexity and feature dependence |
| OPT-34 P2 | Animation/particle LOD | Visual work only; never skip authoritative simulation |
| OPT-35 P2 | Specialized registry dispatch | Extensibility and rule equivalence retained |

No synchronous per-frame GPU occlusion readback. Fail open if depth/bounds/history are uncertain, camera cuts, or near-plane intersections invalidate tests. Zero false occlusion is the correctness goal.

Never stop hidden mobs, redstone, fluids or ticks because rendering is culled. Never lower simulation distance invisibly in a comparison. Reversed depth, FP16, fast math, ray tracing, VRS and mesh shaders are research choices only after capability/precision/bottleneck evidence. Compression cannot silently alter Fidelity pixels.

At 4,000 FPS an extra pass can cost more than it saves. Keep a low-overhead path. Test toggles against reference output; retire failed experiments instead of accumulating dormant code. Research sources support techniques, not promised speedup percentages.
