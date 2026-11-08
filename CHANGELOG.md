# Changelog

Records project artifacts and compatibility claims. Entries describe what is
implemented and validated; they are not a claim of vanilla parity unless stated.

## M0 — First useful pixels

- Added a two-crate Rust workspace: `sunset-core` (authoritative world model —
  block registry, chunks, world, and the authored `building-yard` fixture) and
  `sunset` (Bevy 0.20.1 application: window, free-fly debug camera, chunk
  meshing, and an in-game HUD with frame timing and build identity).
- `cargo run -p sunset -- --dev-fixture building-yard` builds and opens a small
  authored block world centered on the origin.

## M1 — A small Creative building loop

- Replaced the free camera with a first-person player controller: mouse look,
  WASD ground movement, jumping, gravity, and solid-block AABB collision.
- Added block targeting (voxel raycast + wireframe selection outline) and
  Creative break/place with a placement restriction (no placing into the player).
- Added a nine-slot hotbar (five `minecraft:*` target blocks) with number-key
  selection.
- Added a fixed 20 Hz authoritative simulation (input → intent → session →
  simulation → interpolated render view), with chunk re-meshing on edits.
- Added a versioned, atomically-written JSON save format (`saves/*.sunset`) and
  automatic reload on launch.
- Authored block colors stand in for target textures; visual parity is
  explicitly unverified. No login, launcher, networking, or asset import.

## M2 — Prove that the foundation can grow

- Separated the block model into shape, opacity, and orientation so non-cube
  blocks can be added without rewriting the world, collision, renderer, or save
  identity.
- Added a non-full-cube slab (`minecraft:smooth_stone_slab[type=bottom|top]`)
  with half-height collision and model.
- Added transparent glass (`minecraft:glass`) rendered in a separate
  alpha-blended pass that does not occlude neighbors.
- Added an orientable oak log (`minecraft:oak_log[axis=x|y|z]`) with distinct
  end faces.
- Changed the save format to store namespaced block ids instead of array indices
  (schema v2), with deterministic (sorted) chunk output.
- Added a renderer-free deterministic replay test and save round-trip test.
- Added local asset loading: supported blocks now render with the real
  `minecraft:*` textures read from a local Minecraft installation's client jar
  (auto-detected, or via `SUNSET_MINECRAFT_DIR`), with a flat-color fallback.
- The remaining architectural probes (native save data, protocol/authority,
  worldgen/numeric) and the shader/post-effect path remain blocked pending
  target artifacts.


