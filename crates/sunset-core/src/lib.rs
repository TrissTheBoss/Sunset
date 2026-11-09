//! Sunset core: the authoritative world model and simulation shared by
//! gameplay, persistence, and (later) protocol adapters.
//!
//! This crate owns **no** windowing, GPU resources, UI widgets, or account
//! secrets (PRD 4.1). It is plain Rust (plus the `glam`/`serde` generic
//! facilities) behind an application adapter; the `sunset` binary crate
//! consumes it and supplies presentation.

pub mod block;
pub mod chunk;
pub mod fixture;
pub mod intent;
pub mod item;
pub mod physics;
pub mod player;
pub mod save;
pub mod simulation;
pub mod world;

pub use block::{
    block_def, has_collision, is_transparent, occludes, occlusion_key, state_by_id, Axis,
    BlockDef, BlockState, Opacity, Shape, AIR, BLOCKS,
};
pub use chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
pub use item::{item_by_id, item_def, item_for_block, item_id, Item, ItemDef, ItemStack, EMPTY_ITEM};
pub use world::World;
