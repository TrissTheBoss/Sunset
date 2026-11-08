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

  **Architectural probes** (PRD 5.1) are recorded as `blocked` pending capture of
  the exact Java Edition 26.3 artifacts: target resources/shaders, native save
  data, protocol/authority, and worldgen/numeric behavior.

## Next playable increment

- **M3 — First Survival progression** (PRD 5.2): verified recipes, tools,
  inventory, furnace processing, and health/damage/drops in a controlled
  fixture world.

## Blockers

- None for M2's foundation work. The four M2 architectural probes are blocked on
  target artifacts (exact Java Edition 26.3 client/server); all target-specific
  fields remain `unverified` in `compat/targets/26.3/manifest.json`.

