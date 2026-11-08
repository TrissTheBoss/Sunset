//! Sunset core: the authoritative world model shared by gameplay, simulation,
//! and (later) persistence and protocol adapters.
//!
//! This crate owns **no** windowing, GPU resources, UI widgets, or account
//! secrets (PRD 4.1). It is plain Rust behind an application adapter; the
//! `sunset` binary crate consumes it and supplies presentation.

pub mod block;
pub mod chunk;
pub mod fixture;
pub mod world;

pub use block::{block_def, is_transparent, state_by_id, BlockDef, BlockState, AIR, BLOCKS};
pub use chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
pub use world::World;
