# Architecture and renderer

Rust owns authoritative behavior, protocol, storage, assets, scheduling and UI state. Original WGSL handles GPU work. wgpu abstracts GPU APIs; Sunset implements the renderer itself.

Simulation and render cadence are separate. Workers read snapshots and return revision-tagged results; stale jobs are rejected. Registries map namespaced persisted identities to dense runtime handles.

Visibility covers terrain, entities, block entities/chests and particles. Greedy merges must preserve light/AO/material/UV interpolation. Custom LOD preserves silhouettes/overhangs with validated transitions. Upscaling is standard in Enhanced with a native fallback; Fidelity stays native.

Rendering never skips authoritative simulation. Performance changes require correctness and measured evidence.
