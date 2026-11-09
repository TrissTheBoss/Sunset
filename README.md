# Sunset

A free Rust implementation of the Minecraft Java Edition client and integrated
single-player game, working toward externally observable vanilla parity with
Java Edition **26.3**.

> Status: **M2 — Prove that the foundation can grow** (see `PROGRESS.md`). This is
> an authored development fixture and a Creative-only building loop, not a claim
> of vanilla parity.

## Prerequisites

- Rust toolchain (the project is developed on `rustc 1.99.1` / `cargo 1.99.1`).
- A desktop OS with a GPU. Validated on the owner's Windows x86_64 machine.

## Launch

```sh
cargo run -p sunset
```

This builds and opens a window showing a small authored block world centered on
the origin, rendered by a custom raw `winit` + `wgpu` renderer (no Bevy).

## Building on this machine

The workspace has two crates: `sunset-core` (the authoritative world model and
simulation) and `sunset` (the raw `winit` + `wgpu` renderer). The dev profile
(`Cargo.toml`) sets `debug = false` and `opt-level = 0` for our crates while
optimizing dependencies, which keeps builds light on memory.

## Controls

| Input | Action |
| --- | --- |
| Mouse look | Look around (while cursor is captured) |
| `W` / `A` / `S` / `D` | Walk forward / left / back / right |
| `Space` | Jump |
| `Left click` | Break the targeted block (or capture the cursor) |
| `Right click` | Place the selected hotbar block |
| `1` … `9` | Select a hotbar slot |
| `F5` | Save the world + player state |
| `F11` | Toggle fullscreen |
| `Esc` | Release the cursor |

A black outline marks the block under the crosshair. On first launch the world
is generated from the `building-yard` fixture; saves are reloaded automatically
on the next launch (`saves/building-yard.sunset`). An on-screen HUD (hotbar,
crosshair, and FPS text) is not yet implemented in the raw renderer.

## Local Minecraft assets

Sunset loads block textures from a local, user-provided Minecraft installation
(the client jar under `.minecraft`), so supported blocks render with the real
`minecraft:*` textures. It auto-detects `%APPDATA%\.minecraft`, or you can point
it elsewhere with the `SUNSET_MINECRAFT_DIR` environment variable. If no
installation is found, it falls back to authored flat colors. Textures are
loaded for reading only; nothing is bundled, redistributed, or written back.

## Current limitations

- Block textures are loaded, but full models/shaders/post-effects are not yet;
  visual parity remains unverified.
- No login, launcher, or networking (intentionally deferred).
- No pause menu yet (pause is a placeholder); save is manual via `F5`.

See `PRD.md` for the full requirements and `PARITY_MATRIX.md` for coverage.
Development process and the milestone commit/push rule are in
`CONTRIBUTING.md`; browseable documentation lives on the
[GitHub wiki](https://github.com/TrissTheBoss/Sunset/wiki).

# Trademark and Copyright Notice
> Sunset is an independent, unaffiliated project and is not sponsored, endorsed, or approved by Microsoft Corporation or Mojang Studios. “Microsoft,” “Mojang,” “Minecraft,” and “Minecraft: Java Edition” are trademarks or registered trademarks of their respective owners.

> This project contains original code and development assets. It does not include or distribute Microsoft or Mojang source code, game files, or textures. Any references to Minecraft or Java Edition describe the project’s intended compatibility target and do not imply ownership, affiliation, or endorsement.
