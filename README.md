# Sunset

A free Rust implementation of the Minecraft Java Edition client and integrated
single-player game, working toward externally observable vanilla parity with
Java Edition **26.3**.

> Status: **M0 — First useful pixels** (see `PROGRESS.md`). This is an authored
> development fixture, not a claim of vanilla parity.

## Prerequisites

- Rust toolchain (the project is developed on `rustc 1.99.1` / `cargo 1.99.1`).
- A desktop OS with a GPU. Validated on the owner's Windows x86_64 machine.

## Launch

```sh
cargo run -p sunset -- --dev-fixture building-yard
```

This builds and opens a window showing a small authored block world centered on
the origin. The first build compiles Bevy and can take several minutes.

## Building on this machine

Bevy's dependency tree (notably the `windows`, `bevy_reflect`, and `bevy_ecs`
crates) needs more compiler stack and memory than the default toolchain provides
on the development machine (16 GiB RAM). Two workarounds are applied:

1. **Low-memory profile** (`Cargo.toml`): the dev profile sets `debug = false`
   and `opt-level = 0`, avoiding out-of-memory crashes.
2. **Bigger compiler stack**: the `windows`/`bevy_reflect` crates overflow
   `rustc`'s default main-thread stack. A patched `rustc` with a larger stack is
   used via the `RUSTC` environment variable, produced with the `tools/patch_stack.rs`
   helper (copy `rustc.exe` to `rustc-bigstack.exe`, patch it, and set
   `RUSTC` to that path).

A machine with more RAM (or a future rustc/windows-crate fix) should not need
these workarounds; they are not part of the product behavior.

## Controls (M0 debug camera)

| Input | Action |
| --- | --- |
| Mouse look | Look around (while cursor is captured) |
| `W` / `A` / `S` / `D` | Move forward / left / back / right |
| `Space` / `Left Shift` | Move up / down |
| `Left click` | Capture the cursor |
| `Esc` | Release the cursor |

An on-screen HUD shows the build identity and live FPS.

## Current limitations

- Authored block materials (`sunset:*` namespace) stand in for target textures;
  visual parity is explicitly unverified.
- No login, launcher, networking, or asset import (intentionally deferred).
- The camera is a free debug camera, not the playable player controller.

See `PRD.md` for the full requirements and `PARITY_MATRIX.md` for coverage.
Development process and the milestone commit/push rule are in
`CONTRIBUTING.md`; browseable documentation lives on the
[GitHub wiki](https://github.com/TrissTheBoss/Sunset/wiki).
