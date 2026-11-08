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

## Next playable increment

- **M2 — Representative building world + target resources** (PRD 3.4): verified
  target block-state/resource definitions, a larger representative world, and
  more of the Creative set (more block states, hotbar polish).

## Blockers

- None for M1. Target artifacts (exact Java Edition 26.3 client/server) are not
  yet captured; all target-specific fields remain `unverified` in
  `compat/targets/26.3/manifest.json`.

