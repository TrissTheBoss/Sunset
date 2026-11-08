# Sunset Product Requirements Document

**Version:** 2.0  
**Date:** 7 November 2026  
**Status:** Replacement requirements and execution baseline; no implementation or compatibility claim  
**Product:** A free Rust implementation of the Minecraft Java Edition client and integrated single-player game  
**Pinned target:** Minecraft Java Edition 26.3, subject to exact artifact capture  
**Intended reader:** The LLM developer implementing Sunset and the product owner reviewing each playable delivery

## 1. Product objective and decision rules

Build toward complete, externally observable vanilla Minecraft Java Edition parity in Rust. Deliver a small, enjoyable, measurable foundation early, then expand it through working gameplay increments without repeatedly replacing the game model.

The priorities, in order, are:

1. **Vanilla parity.** Preserve the behavior, content, presentation, persistence and interoperability of the pinned target. An optimization must not alter required behavior.
2. **Performance.** Achieve good frame pacing, simulation throughput, loading responsiveness and memory use at equivalent behavior and settings. Measure from the first working build.
3. **Fast playable progress.** Produce useful pixels immediately and a narrow building loop next. Keep the program runnable as features are added.

These priorities govern tradeoffs, not a waterfall schedule. Do not wait for the whole game to reach parity before measuring performance or showing a playable build. Limit the number of supported features in an early milestone while implementing and verifying those features carefully.

### 1.1 Development scope and final scope

- An early development build may omit features. Every omission remains visible in the final baseline inventory.
- An implemented feature may be incomplete or unverified, but must be labelled accordingly. It does not count as parity evidence.
- Prefer a small feature implemented to target behavior over a broad approximation that becomes the permanent game model.
- Temporary development fixtures, authored textures and diagnostic interfaces are allowed. Identify them as development content and keep them outside the vanilla baseline behavior.
- Do not downgrade a required vanilla feature to optional because it is difficult, expensive or blocked by an external service.
- When speed conflicts with correctness, reduce the current milestone's breadth. When performance conflicts with correctness, preserve correctness and record the performance problem.

### 1.2 Completion labels

Use these separate labels in reports:

| Label | Meaning |
| --- | --- |
| Visible development build | Real world data renders and the user can inspect it. |
| Playable foundation | The bounded building loop in section 3 works, including save and reopen. |
| Verified feature subset | Named target scenarios pass; all other behavior remains outside this claim. |
| Full baseline candidate | All required target families and interactions are accounted for and their required cases pass. |

“100% vanilla parity” is the product objective. A finite test suite is evidence of compatibility within its stated coverage, not a mathematical proof over every possible world and input sequence. Never report a percentage of complete Minecraft parity from the percentage of currently written tests that pass.

## 2. Target, implementation and platform boundaries

### 2.1 Preserve the selected target

Retain Java Edition **26.3**. Its official release article is dated **15 September 2026** and records resource pack format **97.1** and data pack format **121.0**. The article also documents resource-pack post effects and input changes that must remain in the final inventory. These release facts do not establish protocol identifiers, executable hashes or complete behavior. [S1]

Create `compat/targets/26.3/manifest.json` incrementally. Keep compatibility evidence outside Cargo's disposable `target/` build directory. Record:

- Exact release ID, artifact origin, client/server hashes and capture date.
- Protocol and data version identifiers, registry/report identities and pack format versions.
- Resource/data pack content hashes and ordering for each fixture.
- Relevant runtime, operating system, graphics backend and reference settings.
- The provenance and verification state of every captured field.

An unknown value is `unverified`, never a guessed verified constant. Generic windowing and development-fixture work may begin while exact artifacts are unavailable. Target-specific evidence and claims require the corresponding artifact and scenario to be captured. Do not silently select an older version because it is easier, and do not chase newer releases during this target's implementation.

### 2.2 Rust boundary

All Sunset gameplay, client behavior, integrated server, protocol adaptation and persistence implementation must be Rust. Established Rust libraries may provide generic facilities. Operating system APIs, drivers, ordinary native dependencies and GPU shader languages are permitted implementation infrastructure; document material native dependencies.

Do not implement the integrated game by launching, embedding or delegating gameplay to Minecraft's Java implementation. An independently running vanilla server is allowed as a test reference and multiplayer interoperability target. Playing against it does not complete Sunset's Rust integrated server.

Implement original code and distribute only assets and dependencies that Sunset may distribute. Import authorized user-provided game resources read-only. The initial authored fixture needs no Minecraft files or account; it makes no claim to be an authenticated Minecraft installation. Real service access must use legitimate authentication and entitlement flows.

### 2.3 First supported environment

Validate one desktop platform first: the owner's actual development machine. If no platform is supplied, use Linux x86_64 as the initial implementation default, then record the real CPU, GPU, driver, RAM and graphics API when available. Do not treat a headless CI machine as a validated desktop or invent reference hardware.

Windows, macOS and additional GPUs follow through a declared support matrix. Platform expansion must not delay the first local playable build. All compatibility claims name the tested platform and target.

## 3. First visible build and first playable foundation

### 3.1 M0 — First useful pixels

**Player outcome:** open the application, see a small block world and move a camera around it.

Implement one application binary, a minimal world model, a window, input, a camera and a simple chunk mesh. Geometry must be produced from the same block/chunk representation that subsequent gameplay will use. A disconnected renderer demo or a pre-rendered scene does not satisfy this milestone.

Use a small authored fixture centered around the origin, with several chunks, recognizable colored materials and a few raised platforms. Temporary authored materials are sufficient. A movable debug camera is acceptable at this milestone only.

Acceptance:

1. One documented command builds and opens the scene.
2. Mouse look and camera movement work; cursor capture can be released.
3. Resize and normal window close work.
4. Basic frame timing and build identity can be inspected.
5. No account login, custom launcher, full feature inventory, asset-import wizard or network connection is required.

Use the first implementation increment for this deliverable. Toolchain setup may affect elapsed time; do not replace the deliverable with a calendar promise. Complete only the minimum design needed to render real core data, and continue target capture alongside it.

### 3.2 M1 — A small Creative building loop

**Player outcome:** enter a bounded world, walk and build something, quit, then return to the same construction.

Initial supported gameplay:

- First-person mouse look, ground movement, jumping and collision with supported solid blocks.
- Block targeting, a visible selection outline and target-compatible Creative breaking/placement for the declared subset.
- A nine-slot hotbar with at least five selectable supported full-cube block states. Empty or unsupported slots must not create fictitious items.
- A small authored multi-chunk world containing flat ground, a wall, raised platforms and positions on both sides of the origin.
- Pause, resume, quit, save and reopen.
- Persistent block edits, player position/orientation and selected hotbar state.

Use `minecraft:stone`, `minecraft:dirt`, `minecraft:grass_block`, `minecraft:cobblestone` and `minecraft:oak_planks` as the initial target-state candidates, verifying their exact state definitions before treating them as target data. Synthetic fallback content uses its own development namespace. Authored colors may still stand in for textures, with visual parity explicitly unverified.

The world is a **development fixture**, not a claim of target terrain generation, a complete Creative mode or Survival mode. Walking and building must already use the durable game-state and authority boundaries in section 4. Do not keep M0's free camera as the playable player controller.

### 3.3 M1 acceptance script

Run and record a repeatable manual or automated script:

1. Start the fixture using the documented launch command.
2. Walk through the origin and across a chunk boundary; look around and jump onto a supported platform.
3. Collide with a wall without passing through it. Verify the relevant placement restriction where a new solid block would intersect the player.
4. Target, remove and place supported blocks; change hotbar selection.
5. Build a small recognizable structure across a chunk boundary.
6. Pause and resume with the implemented local pause semantics.
7. Save and quit normally. Reopen and confirm the construction and supported player state.
8. Replay the same starting state and tick-assigned authoritative input sequence at different rendering rates; authoritative outcomes must match. Test live input separately against the target's sampling and latency behavior.
9. Complete a ten-minute play session without a crash, lost edits or an unbounded work queue.

Record target comparison results separately from functional smoke-test results. M1 functional acceptance and vanilla verification are separate results; passing the former does not imply the latter. A self-authored expected value is not automatically a vanilla oracle. Missing target access leaves the affected behavior `unverified`; the fixture may remain runnable, but M1 must not be advertised as a verified vanilla subset until its declared behavior has evidence.

### 3.4 Intentionally deferred from M0 and M1

Do not make complete authentication, ordinary multiplayer, procedural vanilla terrain, mobs, redstone, fluid simulation, Survival, all camera modes, all menus, full language coverage, advanced resource packs, enhanced anti-aliasing or a self-updater prerequisites for these milestones. Their baseline obligations remain in section 8. The first build needs a direct entry path and a small usable HUD; a branded launcher is not the initial product.

## 4. Durable foundation and modularity contracts

### 4.1 Start with a small workspace

If a working Rust repository already exists, inspect and extend it. Do not rewrite it solely to adopt a preferred framework.

For a new repository, use this starting shape:

| Component | Owns | Must not own |
| --- | --- | --- |
| `sunset-core` | World/state types, registries, numeric compatibility helpers, simulation, local authority, client movement rules where shared, storage and worldgen modules as introduced | Windowing, GPU resources, UI widgets, account secrets or framework scheduling policy |
| `sunset` application | Window/input adapters, client presentation, renderer, UI, audio, loading workflow, later authentication/network adapters | A second authoritative world model or gameplay rules hidden in rendering code |
| `tests/` and `fixtures/` | Compatibility cases, captured expectations, integration tests and repeatable benchmarks | Production dependency on a Java game runtime |

Begin with two production crates, or equivalent modules in an existing project. Extract `protocol`, `storage`, `renderer`, `assets` or test tooling into additional crates when they have real implementations and the boundary improves dependency isolation. Do not create empty crates, a dynamic plugin ABI, an editor, a service architecture or a generic engine framework to demonstrate modularity.

### 4.2 Default presentation stack

For a new repository, default to a pinned stable **Bevy** release for application/window/input/rendering integration and, when needed, UI/audio facilities. Bevy's plugin structure and default facilities provide a practical starting point. This is a delivery choice, not evidence of Minecraft compatibility. [S2]

Keep the authoritative model ordinary Rust behind an application adapter. Bevy entities may represent presentation objects or appropriate dynamic entities; do not make one ECS entity per world block the default storage model. Do not adopt generic rigid-body physics or default PBR materials as vanilla movement, collision or visual semantics.

Keep the graphics adapter replaceable. The early resource/shader probe in M2 must determine whether required target behavior can be supported. If a concrete limitation requires direct `wgpu`, a different Rust renderer or a platform binding, change that boundary through a short recorded decision and retain the gameplay core. `wgpu` provides graphics infrastructure; it does not provide Minecraft's renderer or shader compatibility automatically. [S3]

Pin the Rust toolchain and compatible dependency versions when implementing. Do not invent version numbers in advance or use unpinned repository main branches for the normal build. Avoid a long framework comparison before M0.

### 4.3 State ownership and client/server boundary

Define a typed path:

1. Input produces client intent and presentation updates.
2. Client intent crosses a session boundary.
3. Local authoritative simulation validates and applies relevant actions.
4. The client receives ordered state changes or observations.
5. Rendering consumes a read-only view or bounded deltas and derived caches.

The local session initially runs in-process. It must not need sockets, packet serialization or a second copy of all world data just to imitate a future network. A later remote session implements the target's actual authority, prediction and correction semantics through an adapter. It is not merely the local controller with network I/O attached.

The renderer and UI cannot directly edit arbitrary authoritative blocks or inventory. Derived render state is not saved as world truth. Separate authoritative positions and velocities from visual camera transforms and interpolation.

Keep shared deterministic rules reusable where local authority and client prediction need them, while preserving their different responsibilities. Start with concrete methods and types; add traits only at actual interchangeable boundaries such as world storage or local/remote sessions.

### 4.4 World and content representation

- Use signed world/chunk/section coordinates and test negative-coordinate boundaries.
- Use namespaced resource identifiers plus explicit block-state properties. Internal compact IDs are allowed; do not serialize ephemeral array indices as durable identities without a registry mapping.
- Represent chunks/sections with compact contiguous storage or palettes. A fixed-size fixture must not force the world model to become one permanent dense global array.
- Keep dimension identity, minimum height, height range and other target parameters explicit.
- Separate a block's model, collision shape, selection shape, occlusion, material, light behavior and gameplay behavior. “Solid cube” may describe M1 content, not the universal block interface.
- Use data-driven registries for content and properties where appropriate; implement algorithms in typed Rust. Do not build an arbitrary scripting language or one-off class hierarchy per block.
- World generation, imported chunks and development fixtures must populate the same authoritative model through bounded adapters.

The extension test is practical: adding a new supported block state should not require editing input handling, the renderer's ownership model and the save identity scheme. Introducing a slab or an orientable block must not require replacing the whole world representation.

### 4.5 Time, order, numeric behavior and randomness

Establish these boundaries before expanding M1:

- Run authoritative logic on explicit target tick semantics, independent of rendering. The expected default game tick cadence is 20 Hz; confirm it against the pinned target and record any supported tick controls separately. Never inherit an engine's default fixed-update rate without checking it.
- Define the input sampling, tick assignment, action ordering, pause and catch-up behavior for each implemented feature. Rendering interpolation must not change simulation state or add arbitrary camera latency relative to the target. Framework fixed-step examples are implementation guidance, not Minecraft behavior specifications. [S4]
- Start with a simple, explicitly ordered authoritative schedule. Preserve the target's observable order for neighbor changes, scheduled/random ticks, entities, block entities and transactions as these systems are introduced.
- Deterministic behavior is necessary for many fixtures but is not sufficient for parity. A consistently different sort order is still different behavior.
- Implement tested helpers for relevant Java integer overflow, signed/unsigned shifts, shift-distance masking, conversions, division/remainder and exceptional boundary values. Java's expression specification is a language reference; target captures determine where each operation matters. [S5]
- Preserve required float/double intermediate precision and operation order. Verify math functions or lookup behavior where outcomes depend on them. Do not enable approximate math, reassociation, fused operations or lossy state compression merely to increase throughput.
- Record the RNG algorithm, initialization, ownership and consumption order per subsystem. A seed alone is insufficient. Java's `Random` documentation illustrates why the sequence of calls matters; it does not establish that all Minecraft systems use that class. [S6]
- Sunset-only presentation and diagnostic work must not introduce extra consumption of authoritative RNG streams. Preserve RNG sharing and consumption performed by the target, including calls associated with visual or sound events.

Implement the compatibility primitives needed for the current feature first, and extend their tests with later systems. Do not require a complete Java compatibility library before M0. Test parity-sensitive results in optimized builds as well as debug builds.

### 4.6 Background work and performance structure

Meshing, asset decoding and I/O may run asynchronously when useful. Each job must use immutable or versioned input, bounded queues, and explicit ownership of its result. On completion, check that the chunk/resource still exists and its revision matches the job input. Discard stale results after edits, unloads or newer rebuilds.

Incidental Sunset worker completion order must not introduce unverified changes to authoritative behavior. Commit results through an explicit policy preserving the target's observed availability, dependency and ordering semantics. Record genuine target nondeterminism rather than claiming deterministic equivalence where none has been established. Parallel world generation and simulation are permitted only when their target-visible dependency and ordering behavior is preserved and tested.

Render by chunk/section/material batches as appropriate. Rebuild and upload affected regions, including necessary neighbors, rather than the entire visible world on each edit. Frustum culling and removal of genuinely hidden faces are suitable early techniques. Advanced meshing or occlusion must preserve textures, lighting, transparency and geometry; it is not automatically a safe optimization for every model.

No per-frame clone of the entire world, unbounded task submission, permanent cache growth or blocking file/network I/O on the render loop. These are constraints on a simple implementation, not a demand for a custom scheduler.

## 5. Delivery sequence after the first playable loop

Every checkpoint has a runnable demonstration. Break the larger checkpoints below into small increments, each retaining the earlier playable path. Complete the next increment and report it; continue through the authorized sequence without requiring approval for routine implementation decisions.

| Milestone | Player-visible result | Evidence required for its declared claim |
| --- | --- | --- |
| M0 | Move a camera through a real core-data fixture | Launch, input, rendering and basic frame timing |
| M1 | Walk, build, save and return | Section 3 script, scoped behavior cases, isolated persistence and initial performance report |
| M2 | A more representative building world with actual target resources and varied shapes | Foundation stress cases, a verified narrow subset and early architectural probes |
| M3 | A small complete Survival progression loop | Recipes, tools, inventory, processing and state transitions verified for named content |
| M4 | Ordinary target-server play and expanding locally generated worlds | Independent protocol/authority and worldgen/persistence milestones |
| M5 | Systematic expansion across all remaining vanilla families | Feature and interaction coverage, sustained performance and compatibility testing |
| M6 | Full baseline release candidate | Section 10 release gates across the declared platform and service scope |

M4 is not a promise that multiplayer and world generation will finish together. They are separate work streams after the common foundation. Prefer the next bounded, demonstrable result whose dependencies are ready. Risk probes below begin earlier than full implementations.

### 5.1 M2 — Prove that the foundation can grow

Deliver representative cases rather than hundreds of similar cubes:

- One non-full-cube collision/model case, such as a verified slab.
- One orientable or stateful model and one transparent or cutout material.
- A target-derived block model/texture import path and a small recognizable scene.
- Edits, mesh invalidation and relevant state changes across chunk boundaries.
- Negative coordinates, chunk loading/unloading and stale-job rejection.
- A renderer-free replay of the supported authoritative interaction sequence.
- At least one correctly triggered target sound when resource access permits; retain missing sound coverage explicitly.

Run four narrow architectural probes as separate increments, starting after M0/M1 rather than becoming preconditions for first pixels:

| Probe | Small concrete result | Question it resolves |
| --- | --- | --- |
| Target resources and shaders | Load representative target model/material data and exercise a representative target post-effect path; document any compiler/backend obstacle | Can this presentation stack implement the required resource/shader contract? |
| Native save data | Read a copied target region/chunk and its state mapping; inspect round-trip risks without writing the original | Does the core model preserve target state and unknown fields? |
| Protocol and authority | Capture/decode a target fixture and perform a status exchange with an unmodified target server when available; investigate legitimate login prerequisites | Can the session boundary represent target data and corrections without redesign? |
| Worldgen and numeric behavior | Reproduce a bounded target-derived numeric or generator-stage fixture with its full context; identify unsupported stages | Is there an evidenced route to exact generation rather than a visually similar noise generator? |

A probe may conclude `blocked` or `requires adapter change`. That is a finding, not completion of compatibility. Give it a reproducer, a narrowly scoped follow-up and an explicit owner. Complete the probe by recording its evidence, conclusion and next bounded action; completing the associated subsystem is not its exit condition. Resolve architectural blockers before multiplying content that depends on them. Continue unrelated playable work when a service or artifact is unavailable. Do not turn M2 into full shader translation, world generation, authentication or native-world writing.

### 5.2 M3 — First Survival progression

Implement the following progression as multiple small playable deliveries in a controlled fixture world:

1. Obtain a selected material, pick up its drops and manipulate a small supported inventory.
2. Craft planks, a crafting table and a wooden pickaxe using verified recipes and transactions.
3. Mine supported stone with the correct tool rules; craft a stone tool.
4. Use a supported furnace recipe, fuel and output transaction, including save/reopen.
5. Store items in a supported chest and recover them after reopening.
6. Add the relevant health, damage, death/respawn and drops loop as a separate delivery.

This is a Survival subset, not complete Survival parity. Exact break timing, tool eligibility, item components, stack limits, recipe semantics and state transitions require target evidence. The fixture may place needed resources explicitly; it must not pretend to demonstrate vanilla terrain or loot generation.

Add food, armor, combat, enchantments, effects, experience and additional processing as named subsequent slices. Keep each slice small enough to demonstrate its whole player action and resulting state.

### 5.3 M4 and M5 — Broad compatibility through working slices

For ordinary multiplayer, progress through verified framing/state transitions, legitimate authentication, joining an unmodified target server, receiving world data, movement/correction, block interactions and inventory transactions. Then expand chat/signing, entities, commands, reconnect behavior and the remaining protocol inventory. A status ping alone is not multiplayer support.

For world generation, progress through required data/registries, reproducible generator stages, representative chunks, biome/terrain transitions, structures/features, dimensions and interaction with existing neighboring chunks. Match the full context of the target, not merely the numeric seed. Add native-world write compatibility only after safe preservation and round-trip cases pass.

For gameplay breadth, deliver coherent loops covering entity behavior, fluid updates, automation, travel, dimensions, bosses and late-game progression. Include cross-system and chunk-lifecycle interactions in each feature's acceptance cases. UI, sound, accessibility and target customization expand with their corresponding gameplay; do not leave all of them to an undefined final polish phase.

## 6. Parity evidence and regression contract

### 6.1 Source hierarchy

Use observed behavior of the exact captured target first, then official documentation, independently reproduced data/protocol observations and clearly marked hypotheses. A language specification establishes language behavior, not Minecraft's complete algorithm. An LLM's recalled constant or a plausible tutorial implementation is not target evidence.

Store a compact `PARITY_MATRIX.md` or equivalent machine-readable file plus fixture evidence. Start with the current slice and an inventory of remaining feature families. Expand it continuously. Complete inventory coverage is a full-baseline release gate, not permission to draw the first frame.

Each case records:

- Stable requirement/case ID and feature family.
- Target artifact identity, settings and relevant pack hashes.
- Initial world/player/entity state, coordinates, seed and generation context where relevant.
- Inputs and their order/tick context.
- Expected observations and comparison policy.
- Reproduction command or a precise manual protocol.
- Actual result, tested Sunset revision, evidence location and unresolved uncertainty.

Use `not_started`, `implemented_unverified`, `verified`, `mismatch` and `blocked` states. Applicability is a separate field with reasons. An unknown applicable feature remains a full-parity blocker. Reopen verified cases when target context, implementation or relevant assumptions change.

### 6.2 Comparison policies

- Compare exact discrete gameplay outcomes and ordered events where the target requires them.
- Compare numeric state at the precision and checkpoints required by observable target behavior. Never use one blanket epsilon to hide persistent movement or worldgen drift.
- For visual differences, fix scene/camera/settings and define justified backend-specific tolerances. A different model, missing effect, wrong lighting or incompatible shader is not ordinary rasterization noise.
- For saves, compare semantic state and successful behavior after reload. Compression bytes or irrelevant serialization order need not match; meaningful unknown fields must not disappear.
- For nondeterministic behavior, document the observable invariants or justified distributions and capture uncertainty. Do not normalize away unexplained discrepancies.

Test interactions as well as isolated systems: placement versus collision, inventory versus server correction, updates across chunk boundaries, saving with pending state, and entity behavior during chunk lifecycle changes.

### 6.3 Scope reports

Report counts by feature family and verification state. Show newly discovered gaps, regressions and blocked service coverage. A row labelled “redstone works” or “inventory done” is too broad to establish parity. Passing a locally reduced matrix does not complete the whole target.

## 7. Performance requirements from the first build

### 7.1 Two separate comparisons

**Sunset regression comparison:** compare the same fixture and configuration against a previous Sunset revision, beginning in M0/M1.

**Vanilla comparison:** compare against the pinned target only when the exercised behavior, content, settings, scene and simulation are equivalent. A sparse Rust prototype outperforming complete vanilla Minecraft is not performance-parity evidence. Compare rendering, simulation and end-to-end behavior separately when only some workloads are equivalent.

Performance is engineered alongside correctness. Each new feature receives a relevant cost measurement; broad optimization is not postponed to M6.

### 7.2 Initial budgets

The numbers below are proposed Sunset product budgets, not measured results or Minecraft constants. They apply to a recorded reference machine and fixture. Capture an initial baseline no later than M1. If a budget is inappropriate for the actual hardware, record the evidence and revised budget explicitly before widening the workload; do not silently relax it or fabricate passing results.

| Workload or metric | Initial requirement |
| --- | --- |
| M1 local fixture, 1920×1080, documented camera path | Aim for p95 frame time at or below 16.7 ms and p99 at or below 33.3 ms after warm-up |
| M1 authoritative simulation | Aim for p95 tick execution at or below 5 ms and p99 at or below 10 ms; report scheduling delay/backlog separately |
| Later default-rate simulation workloads | Keep p95 execution within a 40 ms budget for a confirmed 50 ms tick interval, report p99 and overruns, and prevent sustained backlog growth |
| Local edit to visible mesh | Aim for p95 at or below 100 ms in the fixed M1 fixture; separate authoritative commit from mesh visibility |
| Warm launch to interactive M1 fixture | Aim for at most 5 seconds, excluding compilation; measure cold launch separately |
| Memory and queues | Record process memory, GPU memory where measurable, queue depth and cache sizes; repeated fixed-fixture load/unload cycles must settle within a documented bounded cache policy |
| Save/load | Record completion latency and any frame stalls; acknowledged saves must satisfy the durability/recovery contract |

These fixture budgets do not guarantee identical frame rates in every possible world. Do not make all future pathological vanilla worlds fit a universal FPS promise.

### 7.3 Repeatable measurement

For each benchmark, record hardware/driver/OS, graphics backend, toolchain, optimized build revision, power mode, resolution, render/simulation distance, resource/data packs, scenario, entity/block counts, camera/input path and warm/cold state.

For milestone baselines, performance-sensitive changes and suspected regressions, use a fixed warm-up, a fixed 60-second sample and at least three repetitions; record the exact warm-up and expand samples when variance makes a conclusion uncertain. Other implementation increments need only a relevant lightweight timing check; documentation-only changes require no benchmark run. Use uncapped rendering with VSync state reported for throughput, and a separate normal presentation run for frame pacing. Report p50/p95/p99 frame time and tick time, rather than only average FPS. If reporting “1% low,” define its calculation consistently.

Track CPU/GPU bottlenecks, chunk mesh/generation latency, load/save cost, RSS/VRAM when available and queue/backlog behavior. A missing timer is `unavailable`, not zero.

Flag a repeatable deterioration above 10% in a tracked fixture metric for investigation, using the measured noise floor. A flag is not proof of a regression; small differences may be noise. Feature additions must explain their new cost and remain within the relevant budget. Accepted resource tradeoffs need a recorded reason and must preserve parity; do not require an impossible improvement in every metric on every machine.

### 7.4 Full-baseline performance gate

Before broad implementation, name and describe a compact set of future benchmark workloads covering steady rendering, chunk travel, representative entity load, automation/fluid simulation, world generation and save/load. Implement each benchmark when its feature exists; benchmarks for absent systems must not block earlier milestones. Extend the suite using fixed workload definitions as those features become available.

On equivalent supported workloads, aim to match or exceed vanilla's frame-time, simulation and memory performance, and demonstrate a reproducible improvement in at least one meaningful metric. Report per-workload results and tradeoffs; do not hide a serious regression behind an aggregate average. Failure to reach the performance goal does not authorize changing vanilla behavior.

Forbidden shortcuts in the vanilla baseline include skipping required ticks, reducing simulation distance behind the user's setting, omitting entities/effects, simplifying lighting/fluids/AI, changing RNG consumption or presenting lower-quality output as equivalent work.

## 8. Complete vanilla baseline inventory

The following families remain required for the final target. This is a coverage index to expand from target evidence, not a claim that listing a family implements it.

| Family | Required coverage |
| --- | --- |
| World and progression | World options, all target dimensions, terrain/biomes/structures/features, spawning, weather/time, travel and progression |
| Player | Target movement modes, collision, interaction/reach, camera modes, attributes, damage, health/food/armor, effects, experience, death and respawn |
| Blocks and items | All states, models, properties, components, placement/use/breaking, drops, recipes, containers and processing |
| Simulation | Tick controls/order, scheduled/random updates, neighbor notifications, fluids, redstone, pistons, block entities and chunk lifecycle |
| Entities | Spawning/despawning, AI/pathfinding, interactions, combat, projectiles, vehicles, riding, breeding/trading, bosses and events |
| Persistence | Target world metadata, chunks/regions, entities, player state, dimensions, maps/scoreboards, pending state and safe supported version handling |
| Multiplayer | Target protocol, normal authenticated sessions, inventory/entity/world authority, prediction/correction, chat/signing, commands, disconnection and network edge cases |
| Interface | In-game menus/screens/HUD, inventory/containers, settings/defaults, controls, focus/text input, debug interfaces, statistics and advancements |
| Presentation | Models/textures/animation, lighting, transparency, particles, weather/effects, sound/spatialization, fonts/languages and target graphics settings |
| Packs and customization | Required resource/data pack semantics, ordering/reload, errors, target shader/post-effect paths and data-driven behavior |
| Accessibility and services | Target narration/subtitles/accessibility, skins/capes, LAN behavior, social/account functionality and Realms functionality present in the pinned target |

External service access can be blocked or unavailable. Record the dependency and resulting coverage limitation. Do not reclassify a baseline service as optional merely to obtain a full-parity label. Narrower releases may ship with accurate labels while the full objective remains open.

Sunset branding, launcher design, updater infrastructure and development diagnostics are product facilities outside game-behavior parity. Recreating the separate official launcher's appearance is not required. Preserve target launch/configuration functionality needed to play the pinned game.

Third-party mod loaders, Iris/OptiFine ecosystem compatibility, support for other game versions, enhanced AA beyond target behavior, ray tracing, custom gameplay and a modding SDK are optional future work. They must not block or substitute for the vanilla baseline. Shader behavior actually supported by the target remains required.

## 9. Resource, persistence and external-input safeguards

### 9.1 Resources

Keep resource discovery/import small and read-only at first. A configured directory is sufficient for M2; a polished wizard may follow. Report missing or incompatible files precisely. Load only the current feature slice initially, with unsupported resources visible in coverage reports. The final resource contract includes the complete target-required behavior.

Do not assume that target GLSL/resource shaders work simply because the renderer can load WGSL or compile a simple shader. The M2 probe must test representative actual interfaces, inputs and post-effect behavior before declaring the rendering stack adequate.

### 9.2 Development saves and native worlds

M1 may use a versioned Sunset development save format. Give it a distinct extension/directory and store durable state identities plus a schema version. The repository must explain exactly which state survives reload. Add migration or export of this development data as the model evolves; never silently reinterpret old data.

Protect the first saved construction with an atomic commit or equivalent validated save protocol, a previous known-good version and clear error handling. A save is acknowledged only at the documented commit point. Validate process-interruption recovery; describe the filesystem durability assumptions rather than promising immunity to every hardware failure.

Begin native-world support with read-only copies. Native write support requires meaningful round-trip cases, rejection/preservation of unknown data, version checks, recovery tests and backups. Do not overwrite a user's original world during research or silently replace unknown blocks, components or tags with empty defaults. Matching compressed file bytes is not the compatibility objective; preserving state and behavior is.

### 9.3 External inputs and credentials

When their paths are introduced, bound and validate packets, compression, pack archives, paths and resource allocations. Protect tokens using appropriate platform facilities; redact logs and provide sign-out. Do not implement an authentication bypass. Telemetry remains off; diagnostics needed for local development do not require a hosted service.

Implement relevant safeguards with the feature they protect. Do not require a complete updater, service-security architecture or public-distribution checklist before the first local authored fixture.

## 10. Definition of done and release gates

### 10.1 Each implementation increment

An increment is done when:

- Its promised player-visible behavior works through the normal code path.
- Its affected compatibility and failure cases are recorded, with actual results.
- Relevant performance is measured on available hardware, or clearly marked not measured.
- The previous playable workflow remains available.
- New limitations and dependencies are visible; mocks are not reported as finished systems.
- The owner receives the exact launch command, a short demonstration procedure, the result and the next bounded step.

### 10.2 Full baseline candidate

All applicable baseline cases, at every priority/severity, must pass on the declared support matrix. No known gameplay, presentation, protocol or persistence mismatch may be ignored merely because it is low severity. Unknown coverage, unsupported baseline behavior and unresolved service dependencies block claims covering them.

Require complete target-family accounting, interaction coverage, interoperability against unmodified target servers, native-world round trips/recovery, integrated Rust gameplay, usable target UI/accessibility, and the published performance comparison. Confirm dependency/resource provenance and normal authenticated service behavior before distribution.

Publish the exact version, scope, evidence, measured performance and remaining uncertainty. A partial release is allowed and useful; its name and claims must match what it actually implements.

## 11. Instructions to the implementing LLM

### 11.1 Working method

1. Inspect the repository, its instructions and the working build before editing. Reuse functioning infrastructure.
2. Identify the next uncompleted milestone and one small vertical increment within it. State what the player will be able to do when that increment finishes.
3. Capture enough target evidence for that increment's claimed behavior. Record missing evidence without inventing values; continue independent work when possible.
4. Implement the increment through the production world/session path. Do not create parallel mock gameplay to produce an attractive demonstration.
5. Validate the behavior and concrete failure risks it introduces. Avoid tests that merely repeat the implementation or paperwork unrelated to the current slice.
6. Run the relevant fixture and the appropriately scoped timing check or benchmark from section 7.3. Preserve exact results and explain material regressions.
7. Update the small progress/coverage record and deliver the playable result. Continue with the next authorized increment; ask only when an unresolved product choice materially changes the requested outcome or access is genuinely required.

Do not rewrite this PRD into another long plan instead of building. Do not add a framework, broad abstraction, new runtime, mandatory service or unrelated enhancement without a concrete need in the current milestone. Dependency extraction is a means to preserve behavior and ownership, not a milestone by itself.

### 11.2 Minimum working records

Maintain these as small living files, introduced when needed:

- `README.md`: prerequisites, launch commands, playable controls and current limitations.
- `compat/targets/26.3/manifest.json`: captured target facts and verification states.
- `PARITY_MATRIX.md` or equivalent data: feature/case status and evidence links.
- `PROGRESS.md`: completed checkpoint, next playable increment and actual blockers.
- Benchmark records with fixture/configuration/revision and results.
- A short architecture decision only when a material boundary or dependency choice needs explanation.

Do not complete every record before drawing the first scene. Keep user-facing gameplay screens free of implementation terminology; diagnostics belong in developer views and reports.

### 11.3 First assignment

Start with **M0**, then proceed to **M1**. The first assignment is complete only when there is a real rendered core-data fixture; the first playable assignment is complete only when the building-and-reopening loop works.

Expected launch convention for a new repository:

```sh
cargo run --release -p sunset -- --dev-fixture building-yard
```

This is a proposed interface to implement, not a claim that the command currently exists. For an existing project, preserve its working command and document the equivalent behavior.

Before expanding into broad Survival, demonstrate that one non-full-cube shape, target resource input, negative coordinates, chunk boundaries and save identity can be handled without replacing the foundational model. Complete the M2 probes with evidence, conclusions and next bounded actions, and keep their limitations explicit. An unresolved limitation blocks its dependent features, not independent playable slices.

The owner's desired experience is steady visible progress: a world on screen, then a building loop, then recognizable vanilla systems that keep accumulating on the same tested foundation.

## 12. Reference sources

These sources support the limited facts and library capabilities cited above. They are not a substitute for capturing the exact target executable and its behavior. Library APIs must be checked against the versions selected during implementation.

- **[S1]** Mojang, [Minecraft Java Edition 26.3 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3).
- **[S2]** Bevy, [Plugins and application facilities](https://bevy.org/learn/quick-start/getting-started/plugins/).
- **[S3]** gfx-rs, [wgpu documentation](https://wgpu.rs/doc/wgpu/).
- **[S4]** Bevy, [Physics in a fixed timestep example](https://bevy.org/examples-webgpu/movement/physics-in-fixed-timestep/).
- **[S5]** Oracle, [Java Language Specification, expressions](https://docs.oracle.com/javase/specs/jls/se25/html/jls-15.html).
- **[S6]** Oracle, [java.util.Random specification](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/util/Random.html).
