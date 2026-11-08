# Changelog

Records project artifacts and compatibility claims. Entries describe what is
implemented and validated; they are not a claim of vanilla parity unless stated.

## M0 — First useful pixels (unreleased)

- Added a two-crate Rust workspace: `sunset-core` (authoritative world model —
  block registry, chunks, world, and the authored `building-yard` fixture) and
  `sunset` (Bevy 0.20.1 application: window, free-fly debug camera, chunk
  meshing, and an in-game HUD with frame timing and build identity).
- `cargo run -p sunset -- --dev-fixture building-yard` builds and opens a small
  authored block world centered on the origin.
- Authored `sunset:*` block materials stand in for target textures; visual
  parity is explicitly unverified. No login, launcher, networking, or asset
  import is included.
