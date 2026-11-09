# Sunset Modules and pack support

## Extension tiers

Data definitions handle blocks/states/items/particles/models/recipes/tags/biomes/sounds/UI/loot where possible. Namespaces avoid collisions; validate graph cycles/references/property limits/dependencies before activation.

Behavior modules use a proposed Rust-authored Wasm interface with explicit capabilities, memory limits, fuel/deadlines and deterministic host functions. Select the runtime only after safety/platform/performance tests. Batch queries and command buffers; avoid per-voxel host crossings. No ambient filesystem/network access.

Trusted source-build SDK extensions can replace deep engine systems. Rust dynamic-library ABI is not assumed stable. Pin SDK/toolchain and rebuild. Native extensions have application privileges and are not sandboxed; isolate modded profiles. “Every game domain moddable” does not imply bypassing the security boundary.

| Surface | Mechanism | Required independent demonstration |
| --- | --- | --- |
| Blocks/states/items/components | Schemas and behavior hooks | New stateful block/item |
| Entities/AI/collision | Components, ordered systems, queries | Bounded custom mob |
| Particles/material/render | Descriptors and WGSL pass API | Custom effect |
| Worldgen/biomes/dimensions | Seeded generator interface | Repeatable custom dimension |
| Recipes/loot/tags/advancements | Data pipeline | Override and rollback |
| Commands/admin/permissions | Typed registration | Completion and denied-permission tests |
| Inventory/menu/HUD/input | Layout/event API | Keyboard-accessible menu |
| Audio/language/fonts | Asset/event registries | Positional sound and translation |
| Networking | Optional negotiated namespaced channels | Modded-peer handshake |
| Persistence | Namespaced versioned records | Uninstall/reinstall/migration recovery |
| Tick/rules/scheduler | Ordered replacement interfaces | Deterministic rule replacement |
| Renderer/storage/transport internals | Trusted source SDK | Compile-and-run adapter replacement |

Document ownership/lifecycle/ordering/limits/cancellation/threading/stability for every surface. Examples become CI contract tests. Resolve handlers once, batch events, distinguish cancellable pre-events and immutable post-events. Stable priority/module ID order; dependency cycles fail. No reentrant arbitrary mutation. Performance diagnostics attribute CPU/GPU/memory by module. Budget exhaustion produces explicit isolation/error behavior without corrupting state.

## Pack promises

| Format | Scope |
| --- | --- |
| Vanilla resource packs | Exact pinned model/texture/font/language/sound/animation/overlay/precedence behavior |
| Vanilla data packs | Exact pinned registries/functions/tags/predicates/recipes/loot/reload semantics |
| Sunset WGSL packs | Native typed pass/resources/settings API and feature tiers |
| Iris/OptiFine shader packs | Separate GLSL/frontend and rendering-convention compatibility milestone |
| Existing Java mods | No binary execution promise |

WGSL cannot directly execute arbitrary GLSL packs. Translation alone does not reproduce preprocessing, uniforms, attributes, stage order, OpenGL state, buffers and loader conventions. M2 prototypes a Rust-managed frontend on a small legally usable corpus. Maintain construct/feature/pack coverage. If full compatibility requires another backend or forbidden dependency, record an ADR and resolve the scope; do not relabel native WGSL support as existing-pack compatibility. REQ-14 remains incomplete for unsupported required scope.

Do not copy Iris/Horizon code just because it is visible. Use public format documentation and independent tests; review library licenses explicitly. Pack reload validates/stages then atomically swaps or preserves the old state. Zip/decompression/path limits apply.

Normal-server sessions reject gameplay-changing local modules; never alter reach, movement, inventory or packet policy. Server packs follow legitimate consent/settings. Cosmetics must not reveal unreceived data. Modded saves remain isolated.
