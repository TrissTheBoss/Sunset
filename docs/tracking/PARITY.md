# Parity coverage

Denominators come from `cargo xtask inventory` (T-006); until then they are UNKNOWN. Domains and milestones per [02](../02-Parity.md). Counts never shrink silently: new inventory entries increase the denominator.

Target lock: 26.3 (protocol 777, resource pack 97, data pack 121, asset index 34).

| Domain | Required (inventory) | Inventoried | Implemented | Differential verified | Release verified | Milestone |
| --- | --- | --- | --- | --- | --- | --- |
| CORE | UNKNOWN | 0 | 0 | 0 | 0 | M3–M4 |
| RENDER | UNKNOWN | 0 | 0 | 0 | 0 | M3, M5 |
| WORLD | UNKNOWN | 0 | 0 | 0 | 0 | M4 |
| PHYS | UNKNOWN | 0 | 0 | 0 | 0 | M4 |
| TICK | UNKNOWN | 0 | 0 | 0 | 0 | M4–M5 |
| ENTITY | UNKNOWN | 0 | 0 | 0 | 0 | M5 |
| DATA | UNKNOWN | 0 | 0 | 0 | 0 | M5 |
| CMD | UNKNOWN | 0 | 0 | 0 | 0 | M5 |
| SAVE | UNKNOWN | 0 | 0 | 0 | 0 | M4 |
| NET | UNKNOWN | 0 | 0 | 0 | 0 | M6 |
| PACK | UNKNOWN | 0 | 0 | 0 | 0 | M5–M6 |
| UI | UNKNOWN | 0 | 0 | 0 | 0 | M3, M7 |
| AUDIO | UNKNOWN | 0 | 0 | 0 | 0 | M7 |
| LIFE | UNKNOWN | 0 | 0 | 0 | 0 | M3–M7 |

Row-level detail moves to `docs/tracking/parity/<DOMAIN>.md` once inventory exists. Row fields: item ID, target hash, required cases, oracle inputs/hash, comparator/tolerance, implementation commit, test IDs, run IDs, status, discrepancy issue and reviewer.

Enhanced LOD/upscaling and shader-pack extensions have separate quality/capability matrices; they cannot inflate vanilla coverage. This index is not a claim of complete enumeration.

## Mapping from revision 1 IDs

| Revision 1 | Domain now |
| --- | --- |
| PAR-01 Registries/content | CORE (IDs) and DATA |
| PAR-02 World generation | WORLD |
| PAR-03 Blocks/lighting | WORLD (light) and TICK (fluids, updates) |
| PAR-04 Redstone | TICK |
| PAR-05 Player | PHYS and ENTITY (combat, inventory, effects) |
| PAR-06 Entities | ENTITY |
| PAR-07 Rules/content systems | DATA |
| PAR-08 Commands | CMD |
| PAR-09 Administrative objects | CMD |
| PAR-10 Protocol | NET |
| PAR-11 Saves | SAVE |
| PAR-12 Resources | PACK and RENDER |
| PAR-13 Data packs | PACK |
| PAR-14 Fidelity rendering | RENDER |
| PAR-15 UI/input/accessibility | UI |
| PAR-16 Audio | AUDIO |
| PAR-17 Lifecycle | LIFE |
