# Parity standards

## Reference lock

M0 produces `target-lock.json` (private) and `target-lock.public.json` (committed). Public identifiers can be committed; restricted extracted data and proprietary bytes stay private under the private root (default `../sunset-private/`, configurable via `SUNSET_PRIVATE_ROOT`). Reference changes invalidate affected evidence.

```json
{
  "schema": 1,
  "edition": "java",
  "version": "26.3",
  "client_jar_sha256": "<hex>",
  "server_jar_sha256": "<hex>",
  "asset_index_id": "<id>",
  "asset_index_sha256": "<hex>",
  "protocol_version": 0,
  "resource_pack_format": 0,
  "data_pack_format": 0,
  "java_runtime": "<vendor and version>",
  "launch_flags": ["<flag>"],
  "oracle_adapter_version": "<semver or NONE>",
  "provenance": { "source": "local launcher install", "recorded_at": "<RFC 3339>", "tool": "xtask doctor <version>" }
}
```

Every field is read from local artifacts, never typed in by hand. A field that cannot be determined is `null`, and the doctor reports BLOCKED for the dependent suites.

## Inventory sources

In order of preference:

1. **Vanilla data generator reports**, produced locally by the pinned server jar (`--reports`). These cover blocks/states, registries, commands, items and packets where the target version emits them. This is the primary, structured and official inventory source. Record the exact invocation in `target-lock.json`.
2. Pinned asset index, blockstates, models, languages, sounds and fonts from the local client assets.
3. Built-in vanilla data packs from the local server jar (recipes, loot, tags, worldgen JSON, advancements).
4. Black-box observation through the oracle (below) for behavior with no data representation.

Inventory every registry/state/component/packet/command/biome/dimension/recipe/tag/model/setting/save category this way. `cargo xtask inventory` writes counts per domain into [PARITY](tracking/PARITY.md); these counts are the denominators. Representative examples speed up development, but they do not replace inventory accounting. Assess publication of extracted tables separately (see [14](14-Security-Assets.md)). Do not publish decompiled source or copy target source into Sunset.

## Parity IDs

Format: `PAR-<DOMAIN>-<NNNN>`. Domains and planned milestones:

| Domain | Scope | Milestone |
| --- | --- | --- |
| CORE | Arithmetic, RNG streams, IDs, palettes, NBT, codecs | M3–M4 |
| RENDER | Fidelity terrain/light/fog/sky/models/HUD visuals | M3 (VS-1 subset), M5 |
| WORLD | Worldgen, noise, structures, decoration, light engine | M4 |
| PHYS | Collision, movement, player rules, vehicles, elytra | M4 |
| TICK | Scheduled/random ticks, fluids, block updates, redstone | M4–M5 |
| ENTITY | Entities, AI, combat, inventories | M5 |
| DATA | Recipes, loot, tags, advancements, predicates, functions | M5 |
| CMD | Commands, selectors, permissions, command blocks, admin objects | M5 |
| SAVE | Region/entity/POI/player/stats storage and upgrades | M4 |
| NET | Protocol states, packets, authentication, prediction | M6 |
| PACK | Resource/data pack formats, precedence, reload | M5–M6 |
| UI | Screens, settings, input, accessibility, localization | M3 framework, M7 |
| AUDIO | Sound events, pitch/attenuation, subtitles, devices | M7 |
| LIFE | Pause, tick rate, reload, dimension and network transitions | M3–M7 |

One ID per independently verifiable behavior or inventory item. Tests reference the IDs in their `parity_ids` field (see [07](07-Automated-Testing.md)).

## Comparison classes

| Comparison class | Standard |
| --- | --- |
| Discrete | Exact canonical values and observable ordering |
| Numeric | Target arithmetic and observable outcomes; no blanket epsilon |
| Network | Valid encoding, decoded semantic/session-state equivalence and legal timing |
| Visual | Stable same-backend pixel goldens plus calibrated regional cross-renderer metrics |
| Audio | Events, pitch/gain/position/timing and appropriate signal comparison |
| Enhanced | Separate quality/performance contract, not vanilla pixel parity |

Rust must reproduce the relevant Java overflow, shifts, signedness, casts, float widths, rounding, NaN behavior, iteration order and random streams. Use explicit operations (`wrapping_*`, explicit `as` casts with tests, `f32` where Java uses `float`) and independent vectors. Do not improve the RNG or reorder shared draws. Parallel commits and fused operations need an equivalence proof.

Performance boundary: simulation code never uses fast-math, fused multiply-add or reassociation unless equivalence is proven. SIMD in simulation must be bit-identical to the scalar path and tested on both. Rendering-only math (camera-relative positions, packed vertices) may differ from authoritative precision, but never changes authoritative state.

Tolerances need units, scope, calibration and a maximum. Do not widen them to pass. Session IDs, encryption and timing may legitimately differ; compare decoded semantics and normalize only documented nondeterminism. Never normalize ordering bugs.

## Oracle

Feed vanilla and Sunset identical snapshots/seeds/packs/state and tick-indexed actions. Observe identical authoritative phases.

Server-side oracle, preferred order:

1. Vanilla dedicated server with RCON bound to loopback and a per-run random password. Use the target's tick control commands (freeze/step) where the target provides them, and data queries for state.
2. A private Java adapter only when the commands cannot observe the required state. The adapter is original code, kept private, versioned in `target-lock.json`, and independently checked against black-box behavior.

Client-side oracle: rendering and prediction need a client capture path (screenshots with a frozen camera per [08](08-Visual-QA.md), private input-replay adapter). A vanilla server alone cannot validate client rendering or prediction.

Capture the first divergent tick, field path, expected/actual values, action prefix and input hashes. Minimize failing traces automatically. Keep structural diffs behind state hashes. Include pending updates, RNG, entity/inventory state, chunk lifecycle and command context. Sort only semantically unordered data. Verify independently that instrumentation does not alter oracle behavior.

Preserve reference quirks. Fidelity does not fix vanilla bugs.

## High-risk areas

Redstone update order, quasi-connectivity where present, scheduled ticks/pistons/observers and chunk edges; exact worldgen seed mixing/noise/structures/decoration; player collision/stepping/sneaking/swimming/elytra/vehicles/combat/reach; command grammar/completion/permissions/selectors/execute/functions/macros where present/feedback/results; command blocks/minecarts, structure/jigsaw/barrier/light/structure-void/debug/operator objects found in the actual target inventory; pack precedence/reload; region/entity/POI/player/stats/advancement storage and upgrades.

Each high-risk area gets its own risk entry in [RISKS](tracking/RISKS.md) before its milestone starts.

## Certification

NOT STARTED → INVENTORIED → IMPLEMENTED → DIFFERENTIAL VERIFIED → RELEASE VERIFIED. Unit tests alone do not establish parity. UNKNOWN and BLOCKED are not PASS. Regressions and reference changes reopen rows. Every mismatch needs an issue, a minimized reproduction, the affected profile/version and an owner. Time-limited disclosed exceptions cannot count toward unqualified full parity. Publish verified/required counts and untested entries; code size is not progress.
