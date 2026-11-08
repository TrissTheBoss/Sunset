# Parity Matrix

Coverage inventory for Sunset against Java Edition 26.3.

> M0 uses authored development content and makes **no** vanilla-parity claims.
> Every target-specific field is `unverified` until the exact target artifacts
> are captured (see `compat/targets/26.3/manifest.json`).

## States

`not_started`, `implemented_unverified`, `verified`, `mismatch`, `blocked`.

## M0 — First useful pixels

| ID | Case | State | Evidence |
| --- | --- | --- | --- |
| M0-1 | Documented command builds and opens the scene | implemented_unverified | `cargo run -p sunset -- --dev-fixture building-yard` (build + window open smoke-tested) |
| M0-2 | Mouse look + camera movement; cursor capture release | implemented_unverified | `crates/sunset/src/camera.rs` |
| M0-3 | Resize and normal window close | implemented_unverified | Bevy `DefaultPlugins` window handling |
| M0-4 | Frame timing and build identity inspectable | implemented_unverified | `crates/sunset/src/hud.rs` |
| M0-5 | No login/launcher/network required | implemented_unverified | no such code paths present |

M0 acceptance (PRD 3.1) is functionally met: the fixture builds, opens a window,
and runs stably. States remain `implemented_unverified` because no vanilla-target
comparison is claimed at M0.

## Feature families (baseline inventory from PRD 8)

All remaining families (world/progression, player, blocks/items, simulation,
entities, persistence, multiplayer, interface, presentation, packs, services)
are `not_started` at M0.

---

*This file is the compact living record described in PRD 11.2. Fixture evidence
is added as each increment implements and verifies behavior.*
