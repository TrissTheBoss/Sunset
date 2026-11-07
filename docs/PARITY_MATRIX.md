# Sunset target parity and evidence matrix

**Target:** Minecraft Java Edition 26.3, officially released 2026-09-15 ([official release article](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3)). Release identity is verified; exact artifact hashes, protocol identifiers and full behavior inventory await local oracle capture.  
**Status legend:** `U` unverified/needs target research; `N` not implemented; `I` in progress; `P` passed with evidence; `F` failed; `NA` only with documented rationale. Initial status for every row is `U`.  
**Claim rule:** A `P` requires a reproducible case, exact target/build and configuration, evidence artifact/reference, reviewer, and date. A feature cannot be called passed because it compiles or looks plausible. Explicitly designated product enhancements and optional compatibility targets are excluded from the vanilla parity gate; required product enhancements remain mandatory for product release. Unknown applicability blocks a complete parity claim.  
**Severity:** S0 data loss/security/auth bypass; S1 blocks core play or causes major divergence; S2 noticeable feature/behavior gap; S3 minor/UI/cosmetic. S0/S1 block release. Any unresolved applicable S0-S3 baseline mismatch prevents an unqualified 100% parity claim.

| ID | Area / acceptance criterion | Priority | Evidence required / pass condition | Status | Evidence / issue |
|---|---|---:|---|:---:|---|
| TGT-01 | Exact official target release identity, release notes, client/server artifacts and hashes, protocol/version identifiers recorded | P0 | Source register links and hash manifest; no inferred identifiers | U | Pending root research |
| TGT-02 | Target platform/runtime assumptions and feature inventory frozen | P0 | Official sources plus recorded target observations; each feature assigned matrix row | U | |
| TGT-03 | Resource pack format 97.1 recognized; compatibility semantics match target | P1 | Import/load/reject/reload fixtures against oracle | U | Format number verified; behavior pending |
| TGT-04 | Data pack format 121.0 recognized; compatibility semantics match target | P1 | World/data pack fixtures against oracle | U | Format number verified; behavior pending |
| TGT-05 | SDL3/scancode input mappings, persistence and rebinding match target | P0 | Compare supported key/device mappings against oracle | U | Official fact; details pending |
| TGT-06 | Borderless default and 320x240 minimum window size match target | P1 | Fresh settings and resize boundary scenarios | U | Official facts |
| TGT-07 | Relative mouse behavior and transitions match target | P0 | Capture/release across menus, focus loss and resize | U | Official fact; detailed cases pending |
| TGT-08 | Whitelist default and access-control semantics match where applicable | P1 | Fresh config and access-control tests | U | Official fact; scope pending |
| TGT-09 | `/posteffect` custom resource-pack post effects match target | P1 | Pack fixtures, permissions, reload and failure cases | U | Baseline; separate from optional loaders |
| TGT-10 | Full additions inventory, including biome, plants, cushions and straw beds, is covered | P1 | Oracle-derived block/item/worldgen/render/save catalog | U | Categories verified; exact inventory pending |
| IMP-01 | Local user installation discovery and manual import identify target reliably | P0 | Valid, missing, partial and wrong-version fixtures; no game asset downloaded/bundled | U | |
| IMP-02 | Imported files remain unmodified; provenance/missing file message is clear | P0 | Hash before/after; UX capture | U | |
| IMP-03 | Launch, loading, failure and crash return to usable launcher state | P0 | Success/failure scenarios, logs checked for secrets | U | |
| UI-01 | Main/title menu layout, navigation, focus and actions match target | P0 | Screen capture and scripted navigation across declared resolution/scale | U | |
| UI-02 | World list/create/edit/delete flows match target; destructive actions are explicit | P0 | Scenario script; verify persistence and recovery | U | |
| UI-03 | Pause, options, controls, multiplayer and in-game screens match target semantics | P0 | Per-screen interaction checklist and captures | U | |
| UI-04 | Resize/fullscreen/display scaling/cursor capture/text input behave correctly | P0 | Input/display matrix and repro video/log | U | |
| UI-05 | Keyboard navigation and accessible labels work for declared accessibility support | P1 | Keyboard-only path and platform accessibility inspection | U | |
| UI-06 | Localization, missing translation fallback, font and text layout match supported target behavior | P1 | Locale sample and long-string/Unicode cases | U | |
| INP-01 | Key/mouse bindings, conflicts, reset, sensitivity and invert options match target | P0 | Default and rebound control scenarios | U | |
| PHY-01 | Walking, sprinting, crouching, jumping, falling, acceleration and friction match target | P0 | Deterministic test world; input/tick trace and position tolerance defined from target measurements | U | |
| PHY-02 | Collision, step height, slopes, ceilings, edges, doors/trapdoors and corner cases match | P0 | Geometry fixture suite with target reference outcomes | U | |
| PHY-03 | Water, lava, swimming, climbing, flying, elytra/gliding or other applicable movement matches target | P1 | One fixture per applicable mode and transition | U | Applicability pending target feature inventory |
| PHY-04 | Knockback, damage, effects, status changes, respawn and death behavior match | P0 | Single-player and server-authoritative scenarios | U | |
| ACT-01 | Targeting ray, reach, occlusion, selected face and entity targeting match | P0 | Fixed camera/geometry fixtures | U | |
| ACT-02 | Block breaking time, tool/haste modifiers, drops and interruption match | P0 | Tool/block/effect matrix; compare tick-level outcomes | U | |
| ACT-03 | Placement orientation, replaceability, support, collision and interaction rules match | P0 | Placement fixture catalog | U | |
| GAME-01 | Blocks, items, recipes, containers and inventory operations match target | P0 | Target registry inventory + item/container scenario coverage | U | |
| GAME-02 | Crafting, furnace/processing, enchanting, trading and applicable stations match | P1 | Recipe and interaction data corpus; target comparisons | U | |
| GAME-03 | Combat, armor, damage, projectiles, shields and effects match | P0 | Test entity/item scenarios with damage traces | U | |
| GAME-04 | Mobs/entities, AI, spawning, drops, riding, leash and behavior match | P1 | Entity catalog and scripted scenario suite | U | |
| GAME-05 | Redstone, fluids, pistons, containers and tick ordering match | P1 | Component behavior fixtures and multi-tick traces | U | |
| GAME-06 | Weather, day/night, lighting, particles, environmental hazards and events match | P1 | Time-controlled scene comparisons and state outcomes | U | |
| GAME-07 | Dimensions, structures, bosses and progression systems match | P1 | Each target system gets named seed/state scenario | U | |
| GAME-08 | Commands, gamerules, difficulty and other admin/game options match | P1 | Command/option inventory and input/output comparisons | U | |
| WGEN-01 | Same seed/options produce matching biome and terrain outcomes | P1 | Fixed seed corpus; chunk/block-level diff with target versions | U | Exact algorithm/data pending research |
| WGEN-02 | Structures, features, caves, ores and decoration match target | P1 | Seed corpus covering feature cases; locate/compare structures | U | |
| WGEN-03 | Spawn selection, world borders and generation boundaries match | P1 | Boundary/spawn fixtures and coordinates | U | |
| SAVE-01 | New world creation options and metadata match target | P0 | World option fixtures and metadata inspection | U | |
| SAVE-02 | Save/quit/reopen preserves blocks, entities, player and world state | P0 | Repeated cycle fixture; canonical data comparison | U | |
| SAVE-03 | Interrupted save/crash does not corrupt prior recoverable state | P0 | Fault injection at write stages; recovery report | U | |
| SAVE-04 | Save format/version compatibility and unsupported/corrupt world handling are safe | P0 | Valid/old/new/corrupt fixtures; backup and non-destructive failure | U | |
| SAVE-05 | Backup, export and restore workflows reproduce world state | P1 | Round-trip checksums/content diff and UX evidence | U | |
| SAVE-06 | Dimensions/chunks/player/entity persistent data match target | P1 | Fixture-by-fixture persistent-state diff | U | |
| NET-01 | Server address validation, DNS, status, ping and player list match target | P0 | Local test server and malformed/timeout cases | U | |
| NET-02 | Legitimate account authentication/session join works via supported flow | P0 | Authorized test account; never record tokens; successful session trace | U | Auth method pending research |
| NET-03 | Protocol handshake, login, encryption/compression and state transition match | P0 | Official/observed protocol trace and unmodified server interoperability | U | Exact identifiers pending research |
| NET-04 | Play packets cover movement, chunks, entities, inventory, chat and world events | P0 | Per packet family tests against unmodified target-compatible server | U | |
| NET-05 | Chat formatting, commands, suggestions, player list, scoreboard and social UI match | P1 | Server fixture and transcript/UI comparison | U | |
| NET-06 | Disconnect, timeout, reconnect, server errors and protocol mismatch are handled correctly | P0 | Failure injection; no hang or credential leakage | U | |
| NET-07 | Unmodified exact-target servers work without a required plugin/proxy/client mod | P0 | Independent server matrix with configuration recorded | U | |
| AUTH-01 | No online cracked/offline or entitlement-bypass path exists | P0 | Code/config review and negative tests | U | Release blocker if violated |
| RND-01 | Blocks, entities, UI, lighting, sky, weather and particles match target visually/behaviorally | P0 | Screenshot/video comparisons on declared GPU matrix; tolerances documented | U | |
| RND-02 | Graphics options/defaults and display modes match target semantics | P0 | Settings state and output comparisons | U | |
| RND-03 | Unsupported GPU/features fail safely and preserve world/settings | P0 | Forced backend/feature failure cases | U | |
| RND-04 | Vanilla anti-aliasing defaults/options (including no-AA preset if applicable) match | P1 | Default/settings and output comparison on declared GPU matrix | U | Baseline; exact options pending oracle |
| PACK-01 | Resource packs load, order, enable/disable, reload and report errors compatibly | P1 | Test pack corpus with dependency/format/invalid cases | U | |
| PACK-02 | User-imported assets resolve correctly; no proprietary assets bundled | P0 | Clean distribution inventory and provenance audit | U | |
| PACK-03 | Optional third-party shader packs in a declared compatibility subset run/recover | P2 | Named pack/API/GPU matrix; unsupported packs clearly reported | U | Optional enhancement; separate claim |
| SND-01 | Sound events, categories, device selection and spatial behavior match | P1 | Event checklist and audio comparison on declared devices | U | |
| SND-02 | Captions/subtitles/accessibility audio settings match when available | P1 | Settings and scenario evidence | U | |
| SEC-01 | Malformed protocol input is rejected without crash, hang, unbounded allocation or state corruption | P0 | Fuzz corpus, limits, sanitizer/crash results | U | |
| SEC-02 | Malicious resource/shader pack input is constrained and safely rejected | P0 | Adversarial test pack suite and threat review | U | |
| SEC-03 | Credentials never appear in logs, crash dumps, support bundle or telemetry | P0 | Automated redaction checks and manual review | U | |
| SEC-04 | Update authenticity and rollback/recovery behavior meet declared design | P1 | Tampered/interrupted update scenarios | U | |
| PRIV-01 | Telemetry absent by default; any collection is disclosed and controlled | P0 | Network audit and privacy review | U | |
| REL-01 | Long session and repeated world load/save cycles remain stable | P1 | Declared-duration soak results, memory and crash observations | U | |
| PERF-01 | Equivalent scenario metrics recorded for Sunset and vanilla baseline | P1 | Reproducible hardware/settings/scenes; raw metric artifact | U | Baseline budgets pending measurements |
| PERF-02 | No optimization changes gameplay, save, protocol or rendering semantics | P0 | Re-run applicable differential suite after optimization | U | |
| DIST-01 | Release contains only original code/assets and permitted dependencies | P0 | SBOM/license/provenance review; asset scan | U | Legal review pending |
| DIST-02 | Installation, upgrade, uninstall and user-data retention are documented and safe | P1 | Fresh install/upgrade/uninstall scenarios | U | |

## Evidence record template

For each row marked passed, attach or link:

```text
Matrix ID:
Sunset commit/build:
Target exact identifier + artifact hash:
Platform / CPU / GPU / driver / settings:
Scenario, seed, server and preconditions:
Steps and expected target outcome:
Observed Sunset outcome:
Evidence artifact(s) / raw logs (secrets removed):
Result and tolerance:
Reviewer and date:
Known limitation / linked issue:
```

## Release decision

The parity release report must enumerate all `U`, `N`, `I`, `F`, and `NA` rows, with reason and severity. `U` is never implicitly a pass. Any S0/S1 failure blocks release; any unresolved applicable S0-S3 baseline mismatch prevents an unqualified 100% parity claim. The target identity and support matrix must appear in the release notes.

## Detailed target mechanics coverage

These acceptance cases make broad subsystem rows testable. Every applicable case becomes its own issue/test case and must be evidenced. Priority ranks delivery ordering; it does not remove a target feature from a 100% baseline parity gate.

| ID | Acceptance criterion | Priority | Evidence / pass condition | Status |
|---|---|---:|---|:---:|
| MECH-01 | Attack cooldown, critical/sweep attacks, knockback and weapon timing match | P0 | Tick-controlled scenarios across weapon, target, attributes and effects | U |
| MECH-02 | Mining/attack reach and raycast, occlusion, crouch/sprint cases match | P0 | Fixed camera geometry at distance boundaries and occlusion cases | U |
| MECH-03 | Elytra, swimming, climbing and movement-mode transitions match | P1 | Controlled input/velocity traces for start, stop and impact | U |
| MECH-04 | Enchantments, potions, attributes, damage sources and difficulty modifiers match | P1 | Effect stacking/duration, damage, death and attribute fixtures | U |
| MECH-05 | Recipes, processing/furnaces, containers and item component data match | P1 | Target data inventory plus transactions and persistence scenarios | U |
| MECH-06 | Commands, selectors, functions, advancements, loot tables and tags match | P1 | Grammar/output/permission corpus and data-driven fixtures | U |
| TICK-01 | Scheduled/random ticks and block/entity update ordering match | P1 | Controlled state tick traces against oracle | U |
| TICK-02 | Fluid propagation, source/flow behavior and chunk-edge interactions match | P1 | Distance, update order, shape and chunk-boundary fixtures | U |
| TICK-03 | Redstone propagation, signal strength, ordering and edge cases match | P1 | Circuit truth tables and multi-tick component fixtures | U |
| TICK-04 | Piston movement, reactions, ordering and block-entity handling match | P1 | Push/pull/chain/blocking/boundary fixtures | U |
| TICK-05 | Block entities update, render, serialize and synchronize correctly | P1 | Per-type inventory plus update/save/network comparison | U |
| ENT-01 | Entity AI state transitions and target selection match | P1 | Per-family scripted scenarios and state traces | U |
| ENT-02 | Pathfinding, navigation, stuck recovery and collision avoidance match | P1 | Fixed terrain route corpus and movement outcome comparison | U |
| ENT-03 | Spawning/despawning, caps, conditions and chunk unload behavior match | P1 | Controlled time/biome/light/distance/limit scenarios | U |
| ENT-04 | Trading, breeding, taming, riding and passenger behavior match | P1 | Entity interaction, inventory, cooldown and persistence fixtures | U |
| WGEN-04 | Worldgen matches Java RNG, overflow, floating-point and operation ordering semantics | P1 | Oracle vectors and cross-platform deterministic seed corpus | U |
| WGEN-05 | Biomes, structures, dimensions and portals match target | P1 | Seed/coordinate and portal-state comparisons per applicable dimension | U |
| SAVE-07 | Region/chunk records, NBT, lighting and POI data match and recover | P1 | Target fixture read/write, boundary and corruption comparisons | U |
| SAVE-08 | Scoreboards, maps, player metadata and world-level records persist/recover | P1 | Create/update/save/reopen and interrupted-write fixtures | U |
| NET-08 | Signed chat/social reporting, skins/capes match authorized target flows | P1 | Verify target/service conditions; authenticated test accounts, privacy-safe traces | U |
| NET-09 | Realms discovery/join/invitation is classified and implemented or explicitly excluded | P1 | Verify behavior and authorized service access; end-to-end if included | U |
| UI-07 | Pixel/layout, focus order, narration, debug/F3 and gamemode GUI quirks match | P1 | Screen/state inventory, captures and keyboard/focus/narration cases | U |
| RND-05 | Vanilla anti-aliasing defaults/options, including no-AA preset if applicable, match | P1 | Default/settings and output comparison on declared GPU matrix | U |
| RND-06 | 26.3 `/posteffect` resource-pack post effects match | P1 | Pack fixtures; permissions, invocation, reload, ordering and failure cases | U |
| PACK-04 | Optional Iris/OptiFine-style loader compatibility is evaluated separately | P2 | Named loader/version/pack matrix and separate release claim | U |
| EXT-AA-01 | Sunset provides at least one effective non-vanilla anti-aliasing mode documented for its rendering backend | P1 | Required product enhancement; outside vanilla parity gate. Side-by-side capture on declared GPU demonstrates edge smoothing and reports frame-time/GPU impact. | U |
| EXT-AA-02 | Sunset AA setting is separate from vanilla preset, preserves HUD text clarity and has safe unsupported-mode fallback | P1 | Required product enhancement; outside vanilla parity gate. Verify vanilla default against oracle; compare toggles, inspect HUD and exercise unsupported-device/error behavior. | U |

Java RNG/numeric rows are Rust implementation requirements, not permission to ship a Java runtime. Vanilla anti-aliasing and `/posteffect` are baseline target parity. Iris/OptiFine-style loaders are optional and cannot be inferred from the baseline shader/post-effect rows. Unknown target status for Realms/social features blocks full parity until classified.

