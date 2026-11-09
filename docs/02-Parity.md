# Parity standards

## Reference lock

M0 produces target-lock.json: edition/version, client/server artifact hashes, asset index/hash, protocol, pack/data versions, required Java runtime, launch flags, oracle adapter and provenance. Public identifiers can be committed; restricted extracted data/proprietary bytes stay private. Reference changes invalidate affected evidence.

Inventory every registry/state/component/packet/command/biome/dimension/recipe/tag/model/setting/save category through permitted local inspection and original tools. Map each to tests. Assess extracted-table publication separately. Representative examples speed development but do not replace inventory accounting.

| Comparison class | Standard |
| --- | --- |
| Discrete | Exact canonical values and observable ordering |
| Numeric | Target arithmetic and observable outcomes; no blanket epsilon |
| Network | Valid encoding, decoded semantic/session-state equivalence and legal timing |
| Visual | Stable same-backend pixel goldens plus calibrated regional cross-renderer metrics |
| Audio | Events, pitch/gain/position/timing and appropriate signal comparison |
| Enhanced | Separate quality/performance contract, not vanilla pixel parity |

Rust must reproduce relevant Java overflow/shifts/signedness/casts/float widths/rounding/NaN behavior, iteration order and random streams. Use explicit operations and independent vectors. Do not improve the RNG or reorder shared draws. Parallel commits/fused operations need equivalence proof. Camera-relative rendering does not change authoritative precision.

Tolerances need units, scope, calibration and a maximum. Do not widen them to pass. Session IDs/encryption/timing may legitimately differ; compare decoded semantics and normalize only documented nondeterminism. Never normalize ordering bugs.

## Oracle

Feed vanilla and Sunset identical snapshots/seeds/packs/state and tick-indexed actions. Freeze clocks where possible; observe identical authoritative phases. Maintain client and server oracles: a vanilla server alone cannot validate client rendering/prediction.

Capture first divergent tick, field path, expected/actual, action prefix and input hashes. Minimize failing traces automatically. Keep structural diffs behind state hashes. Include pending updates, RNG, entity/inventory state, chunk lifecycle and command context. Sort only semantically unordered data. Independently verify instrumentation does not alter oracle behavior.

Preserve reference quirks. Fidelity does not fix vanilla bugs.

## High-risk areas

Redstone update order, quasi-connectivity where present, scheduled ticks/pistons/observers and chunk edges; exact worldgen seed mixing/noise/structures/decoration; player collision/stepping/sneaking/swimming/elytra/vehicles/combat/reach; command grammar/completion/permissions/selectors/execute/functions/macros where present/feedback/results; command blocks/minecarts, structure/jigsaw/barrier/light/structure-void/debug/operator objects found in actual target inventory; pack precedence/reload; region/entity/POI/player/stats/advancement storage and upgrades.

## Certification

NOT STARTED → INVENTORIED → IMPLEMENTED → DIFFERENTIAL VERIFIED → RELEASE VERIFIED. Unit tests alone do not establish parity. UNKNOWN/BLOCKED are not PASS. Regressions/reference changes reopen rows. Every mismatch needs an issue, minimized reproduction, affected profile/version and owner. Time-limited disclosed exceptions cannot count toward unqualified full parity. Publish verified/required counts and untested entries; code size is not progress.
