# Architecture

## Languages and boundaries

Rust owns simulation, storage, protocol, asset processing, scheduling, UI state, audio orchestration, launcher, mod host and automation. Custom WGSL owns shading, eligible GPU visibility/particles, LOD drawing and upscaling. wgpu is the GPU abstraction, not a prebuilt renderer; winit is the window/input layer. Pin the actual compatible versions after capability tests in M0/M2.

First-party production crates use `#![forbid(unsafe_code)]`. Dependencies and drivers may contain unsafe/native internals; audit that boundary. Safe Rust does not guarantee zero leaks or deadlocks. There is no first-party C/C++; do not claim the OS or drivers have none.

No third production general-purpose language is needed. Java may be used only for private vanilla oracle/adaptation tools, never in Sunset's shipping runtime. GLSL is compatibility input for existing shader packs (REQ-14b, COULD). JSON/TOML are data. A Rust-authored Wasm module platform is proposed after measurement (M6).

## Crates

Create a crate only when it has a real consumer. Planned introduction:

| Crate | Owns | Depends on | Introduced |
| --- | --- | --- | --- |
| `xtask` | Doctor, inventory, runner, bench, visual, report, lint | Workspace crates as needed | M0 |
| `sunset-metrics` | Preallocated frame/tick interval rings, percentiles, counting allocator, counters | none | M1 |
| `sunset-core` | Coordinates, IDs/handles, paletted containers, Java-semantics arithmetic, RNG | none | M2 |
| `sunset-render` | Device, frame loop, arenas, meshers, passes, WGSL | `sunset-core`, `sunset-metrics` | M2 |
| `sunset-jobs` | Worker pool, priorities, revisions, cancellation | `sunset-core` | M2 |
| `sunset-assets` | Read-only local asset resolver, models, atlases, caches | `sunset-core` | M3 |
| `sunset-storage` | Anvil/NBT read (M3), transactions/write (M4) | `sunset-core` | M3 |
| `sunset-sim` | World rules, ticks, collision | `sunset-core`, `sunset-assets` | M3 skeleton, M4 rules |
| `sunset-client` | Binary: window, input, UI, snapshot consumption | all of the above | M2 |
| `sunset-server` | Authoritative scheduling, dedicated binary | `sunset-sim`, `sunset-storage`, `sunset-protocol` | M4 |
| `sunset-protocol` | Transport, codecs, session state | `sunset-core` | M3 harness, M6 full |
| `sunset-modhost` | Capabilities, ordered extension phases | public contracts | M6 |
| `sunset-launcher` | Profiles, provisioning, accounts, updates | `sunset-assets` | M6 |

Rendering cannot mutate simulation. Single-player uses the same server through in-process transport. The render rate never controls ticks. Match the target cadence (normally 20 Hz) and pause/freeze/rate-control semantics.

## Thread model

| Thread | Work | Blocking rules |
| --- | --- | --- |
| Main / render | Window events, input, snapshot consumption, interpolation, visibility, command encoding, submit, present | Blocks only in surface acquire/present (PERF-A03). Takes no lock shared with other threads (PERF-A02) |
| Tick (server) | 20 Hz authoritative simulation, publishes `WorldSnapshot` deltas and `RenderSnapshot` | Never waits for the render thread |
| Workers (cores − 2) | Meshing, light, asset decoding, LOD, compression | Bounded queues, cancellation, stale-result rejection |
| IO | Region reads/writes, pack loading | Atomic writes; bounded buffers |
| Network (M6) | Socket I/O, framing, compression/encryption | Bounded queues to the tick thread |

The tick → render handoff is a lock-free triple buffer (or an atomic pointer swap of an immutable snapshot). The render thread always reads the newest complete snapshot and interpolates with the partial-tick fraction. Mesh results flow from workers to the render thread through a bounded lock-free queue; the render thread uploads them within the per-frame upload budget (PERF-A07).

## Frame pipeline (render thread)

1. Poll events and input; never sleep when uncapped.
2. Acquire the newest snapshot; compute camera and interpolation.
3. Drain at most the budgeted number of finished meshes into the geometry arenas through the staging ring.
4. Section frustum culling (CPU in M2, GPU compaction later if measured) produces a contiguous indirect-draw list per pass.
5. Encode passes in target order: opaque → cutout → entities/block entities → sky/effects → translucent → HUD. One command encoder, one submit.
6. Write timestamp queries into the ring slot; read results from 2 frames earlier.
7. Present. Advance the ring slot (maximum 2 frames in flight).

## Data model

Stable namespaced IDs map to dense runtime handles. Persisted, network and internal IDs are distinct types. Use paletted sections and measured dense/SoA hot data, not an allocated object per voxel. Freeze the registry topology except during validated reload/migration.

Workers consume immutable snapshots. Results carry the world-generation ID, section/neighbor revision and asset epoch; reject stale jobs. Bound queues and caches, prioritize visible work, cancel unloads, reserve presentation/simulation capacity. Mutations commit in deterministic phases. Parallelize only independently computed work with an equivalent commit order.

Use compact vanilla dispatch tables/batches rather than per-block dynamic calls. Hooks sit at meaningful operations with cheap no-listener paths.

## Contracts

Define minimal `WorldSnapshot`, `RenderSnapshot`, `AssetResolver`, `RegistryView`, `TickInput`, `ProtocolSession` and `AutomationSession` contracts when their first consumer appears. Document thread ownership, lifetimes, limits, versioning and errors in `docs/contracts/<name>.md` in the same change. No success-returning stubs.

Reuse bounded scratch/staging buffers. Avoid reference cycles, shared hot locks and raw engine pointers across module calls. Test cancellation, shutdown and reload races with controlled scheduling, not sleeps.

## Dependencies

Candidate list to verify and pin in M0/M2. Each admission records purpose, license, unsafe/native boundary, platform support and version in [decisions](decisions/INDEX.md) or the dependency ledger in PROGRESS:

| Purpose | Candidates |
| --- | --- |
| GPU / window | `wgpu`, `winit` |
| Plain-data casting | `bytemuck` (derive only, no first-party unsafe) |
| Math | `glam` |
| Lock-free handoff / channels | `arc-swap`, `crossbeam-channel` or `crossbeam-queue` |
| Serialization | `serde`, `serde_json`, `toml` |
| Compression / archives | `flate2` with a pure-Rust backend, `zip` |
| Hashing | `sha2`, `blake3` |
| Images | `png` or `image` with limited features |
| CLI / errors | `clap`, `thiserror`, `anyhow` (xtask only) |
| Tracing | `tracing` with compile-time level filtering for hot paths |

Verify every API against the installed version before use (`cargo doc`, crate source in `~/.cargo/registry`). Never write code against a remembered API.
