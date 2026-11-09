# Progress

## Completed checkpoints

- **M0 — First useful pixels.** A window opened showing an authored multi-chunk
  block world and a movable debug camera, with frame timing in the HUD.

- **M1 — A small Creative building loop.** The M0 free camera is replaced by a
  durable player/session boundary: first-person mouse look, ground movement with
  jump and solid-block collision, block targeting (raycast + selection outline),
  Creative break/place, a nine-slot hotbar with the five target blocks, and
  versioned save/reopen.

  - Architecture (PRD 4.3): input → client intent → in-process session → local
    authoritative simulation stepped at a fixed 20 Hz, with an interpolated
    render camera. `sunset-core` owns the player, physics, simulation, and save
    format; `sunset` is the Bevy adapter.
  - Edits re-mesh only the changed chunk (revision tracking).
  - Save format: versioned JSON (`saves/building-yard.sunset`), written
    atomically; reloaded automatically on launch.

  **Validation** (2026-11-08, Windows 11 / AMD Ryzen 5 5600X / Radeon RX 6800 XT,
  Vulkan): builds cleanly (`cargo build`), `cargo clippy -D warnings` and
  `cargo fmt --check` pass, and the "Sunset — M1" window opens and runs stably.
  No vanilla parity is claimed.

- **M2 — Prove that the foundation can grow.** The block model is separated into
  shape, opacity, and orientation, so non-full-cube, transparent, and orientable
  blocks can be added without rewriting the world, collision, renderer, or save
  identity (PRD 4.4 extension test):

  - **Slab** (`minecraft:smooth_stone_slab[type=bottom|top]`) — half-height
    collision and model.
  - **Glass** (`minecraft:glass`) — transparent, alpha-blended, does not occlude.
  - **Oak log** (`minecraft:oak_log[axis=x|y|z]`) — orientable model (distinct
    end faces).
  - **Durable save identity** — saves now store namespaced block ids, not array
    indices (schema v2).
  - **Renderer-free replay** — a headless deterministic replay test of the
    interaction sequence (`simulation::tests`), plus save round-trip.
  - **Local asset loading** — supported blocks render with real `minecraft:*`
    textures read from a local Minecraft install's client jar (auto-detected or
    via `SUNSET_MINECRAFT_DIR`), with a flat-color fallback.

  **Architectural probes** (PRD 5.1): the target-resources probe is partially
  done (textures loaded); native save data, protocol/authority, worldgen/numeric,
  and the shader/post-effect path remain `blocked` pending capture of the exact
  Java Edition 26.3 artifacts.

## Next playable increment

- **M3 — First Survival progression** (PRD 5.2). All five slices landed (items,
  drops, inventory, crafting, tools, furnace, chest, health); see below. The
  remaining work is a UI/HUD to drive the interactions, plus target-evidence
  verification of exact rules.

### M3 progress (items, drops, inventory — first slice)

- Introduced a durable item model (`sunset-core::item`): block items (one per
  block state, placing that block) plus `minecraft:stick` and pickaxe items.
- Added `ItemStack` (item + count, 64-stack limit) and a 27-slot inventory
  (9 hotbar + 18 main) on the player, replacing the block-only hotbar.
- Breaking a block drops its item into the inventory (stacking, then empty
  slots); placing consumes one item from the selected hotbar slot.
- Save schema v3: the inventory round-trips (namespaced id + count per slot).

### M3 progress (crafting + tools)

- Recipe registry with shaped 3×3 matching: log→4 planks, planks→crafting
  table, planks→sticks, planks+sticks→wooden pickaxe, cobble+sticks→stone
  pickaxe.
- Player has a 3×3 crafting grid and a `craft()` transaction (consume inputs,
  add output), persisted in the save.
- Mining tool rules: stone requires a pickaxe and drops cobblestone; other
  blocks drop their own item. New blocks: crafting table, furnace, chest.

### M3 progress (furnace + chest)

- Block entities (furnace, chest) stored in the world by block coordinates.
- Furnace smelts one item per tick (cobblestone→stone, sand→glass) using fuel
  (planks/logs); chest stores 27 items.
- Placing creates the entity; breaking drops its contents then the block item.
- Save schema v4 serializes block entities.

### M3 progress (health, damage, death, drops)

- Player health (max 20), fall damage (accumulated fall distance), death and
  respawn at the spawn point.
- On death the inventory and crafting grid are dropped (cleared) and health
  resets. Health persists in the save.

### Remaining / not yet implemented

- A UI/HUD to drive crafting, furnace and chest interaction (currently the
  authoritative logic is headless-tested; the only in-game interaction is
  hotbar selection + break/place).
- Item *entities* (dropped items lying in the world to pick up) — drops go
  straight into the inventory.
- Tool durability, exact mining speed/break timing, and verified target drop
  tables (require the exact Java Edition 26.3 artifacts).

## Blockers

- None for M2's foundation work. The four M2 architectural probes are blocked on
  target artifacts (exact Java Edition 26.3 client/server); all target-specific
  fields remain `unverified` in `compat/targets/26.3/manifest.json`.

