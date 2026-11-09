//! Sunset core: the authoritative world model and simulation shared by
//! gameplay, persistence, and (later) protocol adapters.
//!
//! This crate owns **no** windowing, GPU resources, UI widgets, or account
//! secrets (PRD 4.1). It is plain Rust (plus the `glam`/`serde` generic
//! facilities) behind an application adapter; the `sunset` binary crate
//! consumes it and supplies presentation.

pub mod block;
pub mod block_entity;
pub mod chunk;
pub mod fixture;
pub mod intent;
pub mod item;
pub mod physics;
pub mod player;
pub mod recipe;
pub mod registry;
pub mod save;
pub mod simulation;
pub mod world;

pub use block::{
    block_count, block_def, block_defs, has_collision, is_transparent, occludes, occlusion_key,
    register_block, state_by_id, Axis, BlockDef, BlockState, Opacity, Shape, AIR,
};
pub use block_entity::{is_fuel, smelt_result, tick_furnace, BlockEntity, Chest, Furnace, CHEST_SIZE};
pub use chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
pub use item::{
    block_drop, is_pickaxe, item_by_id, item_def, item_defs, item_for_block, item_id, register_item,
    DropInfo, Item, ItemDef, ItemStack, EMPTY_ITEM,
};
pub use registry::Registry;
pub use recipe::{craft, register_recipe, Recipe};
pub use world::World;
