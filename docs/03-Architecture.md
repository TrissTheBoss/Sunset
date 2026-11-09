# Architecture

Rust owns simulation, storage, protocol, asset processing, scheduling, UI state, audio orchestration, launcher, mod host and automation. Custom WGSL owns shading, eligible GPU visibility/particles, LOD drawing and upscaling. wgpu is the GPU abstraction, not a prebuilt renderer; winit is a candidate window/input layer. Pin actual compatible versions after capability tests.

First-party production crates forbid unsafe code. Dependencies/drivers may contain unsafe/native internals; audit that boundary. Safe Rust does not guarantee zero leaks/deadlocks. No first-party C/C++; do not claim the OS/drivers have none.

No third production general-purpose language is initially necessary. Java may be used only for private vanilla oracle/adaptation tools, never Sunset's shipping runtime. GLSL is compatibility input for existing shader packs. JSON/TOML are data. A Rust-authored Wasm module platform is proposed after measurement.

| Boundary | Owns | Dependencies |
| --- | --- | --- |
| core-types | IDs, coordinates, semantics | Minimal utilities |
| registry/assets | Validated definitions/resources | core-types |
| simulation | World rules/collision | core-types, registry |
| storage | Saves/snapshots/transactions | core-types, version adapters |
| protocol | Transport/codecs/session state | core-types, version adapters |
| server | Authoritative scheduling | simulation/storage/protocol |
| render | Immutable snapshot presentation | core-types/assets |
| mod-host | Capabilities/ordered extension phases | Public contracts |
| client | Input/prediction/UI/audio | Public simulation/protocol/render interfaces |
| launcher | Profiles/provisioning/accounts/updates | Platform/assets |
| xtask/oracle | Fixtures/tests/reports | Automation interfaces |

Create crates only with real consumers, not empty scaffolding. Rendering cannot mutate simulation. Single-player uses the same server through in-process transport. Render rate never controls ticks. Match target cadence (normally 20 Hz) and pause/freeze/rate-control semantics.

Stable namespaced IDs map to dense runtime handles. Persisted, network and internal IDs are distinct. Use paletted sections and measured dense/SoA hot data; not an allocated entity for each voxel. Freeze registry topology except validated reload/migration.

Workers consume immutable snapshots. Results include world-generation ID, section/neighbor revision and asset epoch; reject stale jobs. Bound queues/caches, prioritize visible work, cancel unloads, reserve presentation/simulation capacity. Mutations commit in deterministic phases. Parallelize only independently computed work with equivalent commit order.

Use compact vanilla dispatch tables/batches rather than per-block dynamic calls. Hooks sit at meaningful operations with cheap no-listener paths. Define minimal WorldSnapshot, RenderSnapshot, AssetResolver, RegistryView, TickInput, ProtocolSession and AutomationSession contracts. Document thread ownership/lifetimes/limits/versioning/errors in Markdown. No success-returning stubs.

Reuse bounded scratch/staging buffers. Avoid reference cycles, shared hot locks and raw engine pointers across module calls. Test cancellation/shutdown/reload races. Dependency admissions record purpose/license/security/platform support.
