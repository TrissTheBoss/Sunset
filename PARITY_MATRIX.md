# Parity Matrix

Coverage inventory for Sunset against Java Edition 26.3.

> M1 uses authored development content and makes **no** vanilla-parity claims.
> Every target-specific field is `unverified` until the exact target artifacts
> are captured (see `compat/targets/26.3/manifest.json`).

## States

`not_started`, `implemented_unverified`, `verified`, `mismatch`, `blocked`.

## M0 — First useful pixels

| ID | Case | State | Evidence |
| --- | --- | --- | --- |
| M0-1 | Documented command builds and opens the scene | implemented_unverified | `cargo run -p sunset -- --dev-fixture building-yard` |
| M0-2 | Mouse look + camera movement; cursor capture release | implemented_unverified | `crates/sunset/src/player.rs` |
| M0-3 | Resize and normal window close | implemented_unverified | Bevy `DefaultPlugins` window handling |
| M0-4 | Frame timing and build identity inspectable | implemented_unverified | `crates/sunset/src/hud.rs` |
| M0-5 | No login/launcher/network required | implemented_unverified | no such code paths present |

## M1 — A small Creative building loop

| ID | Case | State | Evidence |
| --- | --- | --- | --- |
| M1-1 | First-person mouse look + ground movement + jump + collision | implemented_unverified | `sunset-core/physics.rs`, `simulation.rs`; `sunset/player.rs` |
| M1-2 | Block targeting + visible selection outline | implemented_unverified | `physics::raycast`; `renderer::outline_system` |
| M1-3 | Creative break + place with placement restriction | implemented_unverified | `simulation::step` (`BlockAction`); `physics::block_intersects_player` |
| M1-4 | Nine-slot hotbar with five selectable blocks | implemented_unverified | `player::default_hotbar`; `hud.rs` |
| M1-5 | Persistent block edits + player state save/reopen | implemented_unverified | `save.rs` (versioned JSON, atomic write) |
| M1-6 | Fixed-tick authoritative simulation + interpolated view | implemented_unverified | `simulation.rs` stepped at 20 Hz; `camera_system` |

M1 acceptance (PRD 3.3) is functionally implemented: the fixture builds, opens a
window, runs stably, and supports walk/jump/collision, targeting, break/place, a
hotbar, and save/reopen. States remain `implemented_unverified` because no
vanilla-target comparison is claimed.

## M2 — Prove that the foundation can grow

| ID | Case | State | Evidence |
| --- | --- | --- | --- |
| M2-1 | Non-full-cube slab (half-height collision + model) | implemented_unverified | `block::Shape::SlabBottom/Top`; `physics::collision_box` |
| M2-2 | Transparent glass (alpha-blended, non-occluding) | implemented_unverified | `block::Opacity::Transparent`; `renderer::build_chunk_meshes` |
| M2-3 | Orientable oak log (axis x/y/z end faces) | implemented_unverified | `block::end_axis` + `face_color` |
| M2-4 | Durable save identity (namespaced ids, schema v2) | implemented_unverified | `save.rs` |
| M2-5 | Renderer-free deterministic replay | implemented_unverified | `simulation::tests` (replay + round-trip) |
| M2-6 | Target resources/shaders probe | partially done | block textures loaded from a local install (`mcassets.rs`); post-effect/shader path still blocked |
| M2-7 | Native save data probe | blocked | needs a copied target region |
| M2-8 | Protocol/authority probe | blocked | needs an unmodified target server |
| M2-9 | Worldgen/numeric behavior probe | blocked | needs a target-derived generator fixture |

M2's foundation cases are functionally implemented and tested. The
target-resources probe is partially complete: block textures are loaded from a
local Minecraft install. The remaining probes (native save data,
protocol/authority, worldgen/numeric) and the shader/post-effect path are
`blocked` pending capture of the exact Java Edition 26.3 artifacts.

## Feature families (baseline inventory from PRD 8)

The player/movement, block interaction, and persistence families are partially
implemented for M1; all other families (world/progression, entities, multiplayer,
interface beyond the HUD, packs, services) are `not_started`.

---

*This file is the compact living record described in PRD 11.2. Fixture evidence
is added as each increment implements and verifies behavior.*
