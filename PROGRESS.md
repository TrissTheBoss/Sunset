# Progress

## Completed checkpoint

- **M0 — First useful pixels.** A window opens, shows an authored multi-chunk
  block world (`--dev-fixture building-yard`) centered on the origin, and a
  movable debug camera (mouse look + WASD/Space/Shift, cursor capture release).
  Frame timing and build identity are shown in an on-screen HUD. Resize and
  window close work. No login, launcher, or network access is required.

  - Workspace: `sunset-core` (authoritative world model, pure Rust) +
    `sunset` (Bevy application adapter). Presentation stack: Bevy 0.20.1.
  - Chunk meshes are derived from the same block/chunk representation gameplay
    will use; faces are culled against air, including across chunk boundaries.

  **Validation** (2026-11-08, Windows 11 / AMD Ryzen 5 5600X / Radeon RX 6800 XT,
  Vulkan): `cargo run -p sunset -- --dev-fixture building-yard` builds and opens
  the "Sunset — M0" window; the process runs stably. Frame timing and build
  identity are visible in the HUD. Functional smoke test passed; no vanilla
  parity is claimed at this milestone.

  **Build note:** compiling Bevy on this 16 GiB machine needs the low-memory dev
  profile and a large-stack `rustc` (see `README.md` → "Building on this machine").

## Next playable increment

- **M1 — A small Creative building loop** (PRD 3.2/3.3): ground movement with
  collision, block targeting/breaking/placement, a nine-slot hotbar, and
  persistent save/reopen. Replaces the M0 free camera with a durable
  player/session boundary.

## Blockers

- None for M0. Target artifacts (exact Java Edition 26.3 client/server) are not
  yet captured; all target-specific fields remain `unverified` in
  `compat/targets/26.3/manifest.json`.
