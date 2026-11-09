# Architecture — modular content

Sunset separates **content** (data) from **game logic** (typed Rust algorithms).
Every content family is addressed by a dense index plus a namespaced id, and
lives in a runtime `Registry<T>` so that new entries — including mod content —
can be registered without touching the core simulation, renderer, or save logic.

This follows PRD 4.4: *"Use data-driven registries for content and properties
where appropriate; implement algorithms in typed Rust."* It is **not** a plugin
ABI or a scripting language (both explicitly out of scope for the current
milestones).

## The `Registry<T>` pattern

`sunset_core::registry::Registry<T>` (see `crates/sunset-core/src/registry.rs`)
is the one pattern every content family uses:

- `register(name, entry) -> u32` — assigns a dense index under a namespaced name.
- `get(index)` / `find(name)` — look up by index or by name.
- `iter()` — iterate `(index, entry)`.

Content registries are lazily initialised with the core content and held behind
a `RwLock`, so mods can register additional entries at startup. The durable
identity on disk is always the namespaced id, never the index.

## Content families

| Family | Registry | Registration entry point | Status |
| --- | --- | --- | --- |
| Blocks | `block::register_block(BlockDef)` | `BlockDef { id, color, shape, opacity, … }` | Runtime-registered (core + mods) |
| Items | `item::register_item(ItemDef)` | `ItemDef { id, max_stack, block }` | Runtime-registered (non-block items; block items derive from blocks) |
| Recipes | `recipe::register_recipe(Recipe)` | `Recipe { pattern, output }` | Runtime-registered |
| Block entities (furnace/chest) | — | `world.set_block_entity(pos, BlockEntity)` | Data per-position, not a global registry |

Planned (not yet implemented) families that will follow the same pattern:

- **Mobs / entities** — an entity-type registry plus per-entity state.
- **Particles / effects** — a particle-type registry.
- **Dimensions / biomes / structures** — registries of generator stages (see
  PRD 5.3 world generation).

## Adding content

- **A new block**: call `block::register_block(BlockDef { id: "mymod:block",
  color, shape: Shape::FullCube, opacity: Opacity::Opaque, … })` at startup.
  Meshing, collision, and save identity pick it up automatically.
- **A new item**: `item::register_item(ItemDef { id: "mymod:item",
  max_stack: 64, block: None })`.
- **A new recipe**: `recipe::register_recipe(Recipe { pattern, output })`.

## Deferred (out of scope for now)

- A plugin ABI / dynamic library loading (`dlopen`).
- A scripting language for content logic.
- External mod data files (JSON/pack parsing) — currently the registries are
  populated in Rust at startup.

These are deliberate deferrals per PRD §1.1 and §11.1; the registry foundation
is what makes them possible later without a rewrite.
