# Sunset — Product Requirements Document

**Status:** Draft for review; requirements baseline, not an implementation or parity claim  
**Product:** Free, lifetime, pure-Rust reimplementation of the Minecraft Java Edition client and integrated single-player experience  
**Target:** Minecraft Java Edition 26.3, officially released 2026-09-15 — release identity verified; exact artifact hashes, protocol identifiers and complete behavior inventory remain pending oracle capture.  
**North star:** Exact vanilla parity is the release objective (priority 1); performance is priority 2, measured only against equivalent behavior and hardware.  

## 1. Product intent

Sunset is an independently implemented Rust program that lets a player install and run an experience compatible with the selected vanilla Java Edition target. It should support ordinary Java multiplayer, local single-player with a complete integrated server and world lifecycle, and ordinary client customization. It must not require the user to buy Sunset or pay a subscription. “Free for lifetime” means no Sunset purchase, subscription, paid feature tier, or time-limited license; it does not mean that Mojang/Microsoft accounts, Minecraft ownership, services, or third-party services are free.

The product aims to reproduce the target's externally observable behavior, including user interface, input, rendering, simulation, world generation, persistence, protocol interaction, and edge cases. This is an ambition and a release gate, not a statement that parity has been achieved. Compatibility work must use clean-room, original Rust implementation and independently authored assets/code. No proprietary Minecraft code, bundled game assets, credentials, or authentication bypasses.

## 2. Users and jobs

- **Vanilla player:** launch and play the selected version in familiar single-player and multiplayer flows.
- **World owner:** create, explore, quit, reopen, and preserve a world without data loss or unexplained changes.
- **Server player:** connect to ordinary, unmodified Java servers using legitimate account/session authentication and standard server flows.
- **Customizer:** use resource packs, shader packs where supported by the target, accessibility/UI settings, and key/mouse settings.
- **Contributor / release reviewer:** inspect evidence for compatibility claims, reproduce test cases, and identify known gaps.

## 3. Product principles and hard constraints

1. **Parity before optimization.** Optimize only after a baseline behavior is captured; demonstrate no compatibility regression.
2. **Evidence before claims.** Every release compatibility claim maps to a target build, reproducible test, result, and known limitation. Never label an unverified component “100% compatible.”
3. **Legal access.** Use official/authorized authentication and session services when required. Do not impersonate accounts, defeat entitlement checks, or offer offline/cracked access to online services.
4. **No bundled Minecraft assets.** Users supply/import files from a local, legitimately obtained installation. Import must explain provenance and missing files clearly. Sunset may distribute only original assets and dependencies whose licenses allow distribution.
5. **Original implementation.** No decompilation-derived code, copied proprietary source, or redistributable game asset. Behavioral observation and protocol compatibility work must be reviewed for applicable terms and jurisdiction before release.
6. **Data safety.** Preserve worlds and player data. Never silently overwrite or migrate irreversibly; make backups before any required migration and provide recovery guidance.
7. **Ordinary multiplayer.** Work with unmodified target-compatible Java servers under normal authentication and protocol negotiation; no server plugin/mod is a prerequisite for baseline play.
8. **Pure Rust product code.** The product implementation is Rust. Build tooling and platform libraries may be external; any exception that embeds a non-Rust runtime or game implementation requires architecture review and explicit documentation.

## 4. Scope

### In scope

- Desktop GUI launcher and in-game client, including settings, input, menus, HUD, chat, inventory, pause, and accessible control flows.
- Local installation discovery/import and version-specific runtime configuration, without distributing game assets.
- Rendering pipeline and target-compatible visual behavior; anti-aliasing, shader/resource pack selection and loading, graphics settings, and fallback/error states.
- Player movement, collision, camera, interaction, block placement/breaking, entities, items, inventory, combat, status effects, and client simulation.
- Integrated server and local world creation, world generation, gameplay, saving, loading, backup/restore, and lifecycle.
- Java multiplayer transport/protocol, status/ping, login/session/authentication, encryption/compression where required, play state, disconnect and reconnect behavior, and supported server compatibility.
- Sound, language/localization, accessibility, input devices, data/resource packs, and other target-observable client functions.
- Installation/update UX for Sunset itself, crash handling/logging, telemetry-off-by-default (if any), and user data management.

### Out of scope unless later approved

- Requiring a custom server, proxy, plugin, mod, or modified client to play vanilla multiplayer.
- Shipping Minecraft binaries, assets, sounds, fonts, textures, or account tokens.
- A promise to support every third-party mod, mod loader, server fork, platform, or historical game version. These may become separate compatibility targets.
- Reverse engineering or use of unauthorized source/assets. Any research technique or distribution plan with legal uncertainty must be reviewed before use.
- Guaranteed access to Mojang/Microsoft-operated services; service availability and account eligibility are external dependencies.

## 5. Release target and parity definition

The pinned target is Minecraft Java Edition 26.3, announced in the [official release article](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3) on 2026-09-15. Before behavior is frozen, record the exact client/server artifacts and hashes obtained through authorized means, protocol/version identifiers, supported OS/runtime assumptions, and complete feature inventory. The official article identifies resource pack format 97.1 and data pack format 121.0; these values are verified, while their full semantic coverage must be captured from the target oracle. The article also identifies SDL3/scancode input changes, borderless default behavior, relative mouse behavior, a 320x240 minimum window size, whitelist enabled by default, custom resource-pack post effects through `/posteffect`, and additions including a biome, plants, cushions, and straw beds. Each must be mapped to acceptance cases without treating this summary as a complete changelog. The target may change only through a documented decision. No guessed protocol values, worldgen parameters, block/entity identifiers, timing constants, or feature list may be presented as verified.

**Parity** means the same externally observable result for a defined scenario under comparable inputs, target configuration, world state, and server conditions. It includes successful behavior and failure behavior. Test oracle hierarchy: (1) target behavior observed with documented setup; (2) official documentation/release notes/specifications; (3) independently reproduced protocol or data behavior; (4) clearly marked inference. Differences in platform-specific rendering or timing need an explicit tolerance and must not mask gameplay-visible discrepancies.

**100% vanilla parity gate:** all applicable vanilla baseline target-parity rows in `PARITY_MATRIX.md`, at every priority, have evidence and pass on the declared support matrix; no unresolved S0/S1 mismatch or data-loss issue; all target features are classified and tested; multiplayer interoperability is demonstrated against unmodified servers; world save/load and migration safety pass. Explicitly designated product enhancements and optional compatibility targets are outside the vanilla baseline; required product enhancements remain mandatory for product release. Unknown applicability cannot be treated as optional.

## 6. Functional requirements

Priority meanings: **P0** required for any credible playable release; **P1** required for parity release; **P2** release polish / broad platform support. Priorities do not waive target parity: P1 items can block a parity claim even if deferred from an early playable build.

### 6.1 Import, launcher and lifecycle

- **FR-001 (P0):** Discover user-provided installations or allow an explicit local directory/file selection. Validate required files against a target manifest; show missing, invalid, or ambiguous components without downloading bundled game assets.
- **FR-002 (P0):** Keep each imported installation/version isolated; let user select launch configuration and report exact target identity used.
- **FR-003 (P0):** Present clear launch, loading, failure, crash, and return-to-launcher states. Logs are user-accessible and redact secrets.
- **FR-004 (P1):** Reproduce target launcher-visible version selection and relevant startup configuration behavior, to the extent needed for the declared target.
- **FR-005 (P1):** Updates to Sunset are signed/verified where supported, atomic, recoverable, and never overwrite imported Minecraft files.

### 6.2 Authentication and multiplayer

- **FR-010 (P0):** Support legitimate account authentication and session joining through authorized flows. Protect refresh/access tokens using platform security facilities where available; never log tokens; allow sign-out/revocation.
- **FR-011 (P0):** A user can enter a server address, resolve/connect, observe status/ping, join with normal authentication, receive disconnect reason, and leave cleanly.
- **FR-012 (P0):** Implement the target's protocol state transitions, framing, compression, encryption, keepalive, packet handling, and error paths as verified against official/observed target behavior. Exact protocol semantics pending research.
- **FR-013 (P1):** Interoperate with unmodified servers running the exact target and supported compatible versions, including common proxy/network conditions. Test server variants are evidence only, not additional guarantees.
- **FR-014 (P1):** Chat, commands, tab completion, player list, status effects, inventory/entity/world updates, and other multiplayer-visible behavior match the target and server authority.
- **FR-015 (P0):** No cracked/offline identity path may join online servers or evade account/entitlement checks. Offline local single-player is allowed only as defined by target behavior and must not masquerade as authenticated multiplayer.

### 6.3 Integrated server and worlds

- **FR-020 (P0):** Create local worlds using the target's applicable world options; enter, pause, save, quit, reopen, and continue without loss or duplication.
- **FR-021 (P0):** Implement integrated server simulation and client/server boundaries so local gameplay matches target behavior, including tick ordering and authoritative outcomes where observable.
- **FR-022 (P1):** Match target terrain, biome, structure, feature, and spawn generation for equivalent seed/options. Exact algorithms, data and constants must be verified; no guessed parity claim.
- **FR-023 (P1):** Match target world/chunk persistence, player/entity data, metadata, dimensions, and recovery semantics. Preserve compatible saves; any format transition is backed up and reversible or explicitly opt-in.
- **FR-024 (P0):** Save safely on normal quit and recover from interrupted writes. Provide explicit backup/export and restore workflows. Never silently clobber a world on parse/version errors.
- **FR-025 (P1):** Match world options, difficulty, game rules, commands, LAN sharing if applicable to target, and world lifecycle edge cases.

### 6.4 Gameplay, physics and interaction

- **FR-030 (P0):** Support first-person/third-person camera modes provided by the target, mouse/keyboard movement, jump, crouch/sprint, look, and configurable bindings.
- **FR-031 (P0):** Match collision shapes, movement acceleration/friction, gravity, jump, stepping, swimming/flying/climbing/falling, knockback, and environmental interactions. Constants are version-specific and remain pending verification.
- **FR-032 (P0):** Match block targeting, reach, breaking progress, tool behavior, placement rules, use/interact behavior, and prediction/correction in multiplayer.
- **FR-033 (P0):** Implement blocks, items, entities, recipes, inventories, containers, crafting, combat, damage, health/food/armor, effects, experience, death/respawn, and drops required by the target.
- **FR-034 (P1):** Cover redstone/automation, fluids, projectiles, vehicles, mobs/AI, boss/event behavior, dimensions, and other target systems with parity evidence.
- **FR-035 (P1):** Match tick, random, and ordering behavior where it affects reproducibility, simulation, or multiplayer synchronization.

The gameplay inventory is decomposed for implementation and verification. “Covered” means each applicable subfeature has a target scenario, not merely a subsystem smoke test. Movement cases include attack cooldown, reach/raycast selection, crouch/sprint transitions, elytra, swimming and climbing. Survival cases include enchantments, potions, attributes, damage sources and difficulty/game-rule interactions. Item cases include recipes, furnace/processing, containers and version-specific item components. Automation cases include fluid flow, redstone propagation, scheduled/random ticks, piston ordering, block entities and chunk-boundary behavior. Entity cases include AI state, pathfinding, spawning/despawning limits, trading, breeding and riding. Exact mechanics remain oracle-derived.

### 6.5 GUI, settings and accessibility

- **FR-040 (P0):** Provide title/menu, world selection/creation, pause, options, controls, video/audio, multiplayer, chat, HUD, inventory and in-world screens applicable to the target.
- **FR-041 (P0):** Mouse, keyboard, focus, resize/fullscreen, display scaling, cursor capture, and text entry work consistently. Input rebinding must identify conflicts and allow reset.
- **FR-042 (P1):** Match target settings semantics and defaults, including UI scale, FOV, render/simulation distance, graphics, particles, audio, language, captions/accessibility and control settings where present.
- **FR-043 (P1):** Provide keyboard navigation and screen-reader-friendly labels where platform and Rust UI stack allow; communicate limitations and avoid blocking play on pointer-only flows.
- **FR-044 (P1):** Localize all user-facing Sunset strings and support target languages as defined by scope; missing translations fall back predictably.

### 6.6 Rendering, shaders, resource packs and sound

- **FR-050 (P0):** Render target scenes, UI, blocks, entities, lighting, weather, particles, and effects with correct gameplay-visible output. Rendering backend and supported GPUs are declared.
- **FR-051 (P0):** Provide stable graphics settings and graceful unsupported-device/shader errors; preserve the user's world and settings on failure.
- **FR-052 (P1):** Support target-compatible resource packs, pack ordering, enable/disable, dependency/format checks, reload, and clear errors. Do not bundle target assets; local import uses the user-supplied installation.
- **FR-053 (P1):** Support shader packs and anti-aliasing options that the target supports, including selection/configuration/reload and safe fallback. Exact format/API compatibility must be researched; no universal shader compatibility claim without tests.
- **FR-053a (P1, Sunset enhancement):** Provide at least one documented, effective anti-aliasing mode beyond the vanilla default, selected for the declared rendering backend (for example FXAA, MSAA or TAA only where technically appropriate). The vanilla-matching preset remains at the target's verified default. Sunset AA controls live in a clearly separate settings namespace; toggling them must not affect gameplay, HUD text clarity must be reviewed, GPU/frame-time cost reported, and unsupported modes must fail safely with an actionable message. This is a required Sunset enhancement, tracked separately from the 100% vanilla-parity gate.
- **FR-053b (P1):** Support baseline target-compatible resource-pack shader/post-effect behavior, including 26.3 `/posteffect`; separately document any shader-pack language/API subset. Third-party loader ecosystem compatibility is a separate optional commitment and cannot substitute for baseline pack behavior.
- **FR-054 (P1):** Match sound event coverage, category volumes, device selection, spatialization and subtitles/captions where available.
- **FR-055 (P1):** Define image comparison tolerances by hardware/backend and separate cosmetic rasterization variance from gameplay-relevant visual mismatch.

Vanilla target graphics configuration, including anti-aliasing defaults/options, is baseline parity. The 26.3 `/posteffect` custom resource-pack post-effect path is also baseline. Third-party shader ecosystems such as Iris/OptiFine-style support are optional compatibility targets unless explicitly committed. These tracks have separate evidence; supporting `/posteffect` does not imply third-party loader support.

### 6.7 Reliability, security and privacy

- **FR-060 (P0):** Validate untrusted network and pack inputs; bound resource use and reject malformed data safely. Treat servers and packs as untrusted.
- **FR-061 (P0):** Never expose credentials in logs, crash reports, screenshots, or support bundles. Crash reports are opt-in and scrubbed.
- **FR-062 (P0):** Crash recovery must preserve world data and provide actionable diagnostics.
- **FR-063 (P1):** No telemetry by default. If later introduced, disclose fields, purpose, retention, and opt-in/out behavior before collection.
- **FR-064 (P1):** Meet declared performance budgets without changing target simulation results, protocol behavior, persistence, or rendering semantics.

## 7. Non-functional requirements and measurable goals

All numerical targets below are project gates to be calibrated on reference hardware, not Minecraft constants. A release declares OS/GPU/CPU/RAM matrix and comparable vanilla setup.

- **Performance (priority 2):** On each reference machine, report cold/warm launch time, time to menu/world, median and 1% low frame rate, frame-time distribution, CPU/GPU/RAM use, chunk generation throughput, save latency, and network overhead. Once parity baseline passes, target at least parity with vanilla in median frame rate and memory for the same scene/settings, then demonstrate a measurable improvement on at least one published metric without regressions elsewhere. Freeze exact numeric budgets after baseline measurement.
- **Reliability:** zero known data-loss defects; world writes are crash-safe by design; soak tests across long sessions and repeated load/save cycles. Release threshold for crash rate is declared after telemetry/privacy decision; absent telemetry, rely on reproducible test evidence and opt-in diagnostics.
- **Compatibility:** publish OS and GPU support matrix; test protocol and save fixtures for exact target; unsupported configurations fail with actionable notice.
- **Security:** threat model covers credential storage, malicious servers, network packet parsing, resource packs/shaders, local imported files, update chain, and crash artifacts. Security issues receive severity and release-blocking disposition.
- **Usability:** first launch/import and first local world are guided; errors identify next action; settings persist; no world can be deleted by an ambiguous destructive action.
- **Maintainability:** modular Rust crates for launcher/platform integration, client, renderer, assets/resources, protocol, integrated server, world storage and shared data. Final crate boundaries may evolve but ownership and tests remain explicit.

## 8. Architecture and implementation constraints

Use idiomatic Rust with clear subsystem boundaries and stable interfaces. Keep authoritative gameplay and protocol state independent of rendering. Version-specific registries/data are explicit and validated. Avoid a design that requires shipping target binaries or assets. Keep imported files read-only unless the user explicitly chooses a documented write location. Use platform APIs through audited Rust bindings as needed; document native dependencies and their licenses. Any non-Rust runtime embedded for functionality must be reviewed against the pure-Rust requirement before adoption.

Suggested subsystems: launcher/import; authentication/session; window/input; UI; renderer/GPU resources; resource-pack/shader loader; audio; client simulation; protocol; integrated server; worldgen; registries/data; save format; diagnostics/update. API boundaries and versioned data manifests should make target verification traceable.

Java-compatible numeric behavior is required wherever observable: worldgen and simulation account for integer overflow, Java RNG behavior, floating-point operations and ordering. Java may be a test oracle only; shipped product logic remains Rust. Save tests include region/chunk records, NBT, lighting, points of interest, player data, scoreboards and maps, plus interrupted-save rollback/recovery. GUI review includes pixel/layout comparisons, focus order, narration, debug screens (including F3 where applicable), and gamemode-specific quirks. Multiplayer inventory explicitly classifies social signing/reporting, skins/capes and Realms as required, conditional, unavailable or optional from verified target behavior and authorized service access; unknown status blocks a complete parity claim.

## 9. Delivery roadmap and gates

Stages are sequential gates, not promises of calendar dates.

1. **R0 — Target and rights baseline:** verify exact 26.3 identity, official facts, artifact provenance/allowed local import model, protocol research constraints, support platforms, and threat model. Gate: reviewed source register; all uncertain facts explicitly marked.
2. **R1 — Vertical slice:** Rust app opens, imports local target files, displays GUI, launches a minimal rendered local scene, reads/writes a test world container safely. Gate: clean-room provenance review and basic crash-safe storage.
3. **R2 — Playable core:** movement/collision, interaction, inventory, core simulation, local world lifecycle and renderer. Gate: recorded golden scenarios against target; no save loss.
4. **R3 — Integrated survival parity:** target gameplay breadth, worldgen, mobs/items/blocks, world persistence and settings. Gate: feature inventory mapped to matrix with evidence.
5. **R4 — Multiplayer:** legitimate auth, protocol, chat, server play, disconnect/reconnect, unmodified-server interoperability. Gate: external server test matrix and security review.
6. **R5 — Customization and polish:** resource/shader packs, anti-aliasing, sound, localization, accessibility, broad platform/GPU matrix. Gate: pack compatibility and failure behavior documented.
7. **R6 — Release candidate:** full baseline matrix pass at every priority, performance report, security/privacy review, licensing/provenance review, recovery test, installation/upgrade documentation. Gate: no open S0/S1 parity or safety issue and no unknown target feature. Optional compatibility rows are excluded only when explicitly designated with rationale. A “100% parity” label requires every applicable baseline row to pass.

## 10. Risks and unresolved decisions

- **Target detail gap:** 26.3 release identity and selected release facts are verified, but artifact hashes, protocol identifiers and full behavior inventory are not yet captured. Do not implement against assumed protocol/behavior.
- **Scope size:** full Java Edition behavior is a major multi-year effort. Stage gates prevent a playable prototype from being confused with parity.
- **Legal/IP:** behavior compatibility, protocol work, trademarks, distribution, and user import can have jurisdiction- and method-specific constraints. Obtain qualified review before public distribution; do not copy code/assets.
- **Auth dependency:** identity/session services are external and may change. Keep authentication modular and follow official supported flows.
- **Parity oracle:** some behavior is undocumented or nondeterministic. Record scenario, seed/state, timing and uncertainty; use differential tests and explicit tolerances.
- **GPU variance:** shader and raster output differs across drivers. Define supported backend matrix and compare behavior separately from exact pixels.
- **World data risk:** format differences or corruption can cause irrecoverable loss. Preserve originals, back up before migration, fuzz parsers and test interrupted writes.
- **Pure Rust interpretation:** system APIs and graphics/audio drivers are native dependencies. Document dependency boundary and obtain product-owner agreement on what “pure Rust” means.
- **Lifetime-free sustainability:** no monetization constraint can limit hosted services. Prefer local operation and make external costs/dependencies explicit; do not promise service availability Sunset cannot control.

Open decisions: supported operating systems at first release; exact account types/auth method; minimum GPU/API; whether shader compatibility means a named standard subset or selected popular packs; telemetry (default none); first support window for prior server versions; world migration policy; numerical performance budgets after baseline.

## 11. Definition of done for this PRD

The PRD is ready for implementation planning when the local oracle manifest records artifact hashes and protocol identifiers, completes the target feature inventory, reviews legal/auth/import assumptions, selects first platform and test hardware, and approves measurable baseline budgets. The official release identity is established; target-specific behavior still needs captured evidence before parity implementation is fully scoped.

