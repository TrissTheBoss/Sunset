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

