//! Chunk storage: a compact, contiguous 16x16xH volume with a monotonic
//! revision counter used for stale-job rejection (PRD 4.6).

use crate::block::{BlockState, AIR};

/// Horizontal extent of a chunk, in blocks.
pub const CHUNK_SIZE: usize = 16;
/// Vertical extent of a chunk, in blocks (world Y `0..CHUNK_HEIGHT` for M0).
pub const CHUNK_HEIGHT: usize = 64;

/// A single chunk of blocks, backed by a compact contiguous slice.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Chunk X coordinate (world X = `cx * CHUNK_SIZE + local x`).
    pub cx: i32,
    /// Chunk Z coordinate (world Z = `cz * CHUNK_SIZE + local z`).
    pub cz: i32,
    blocks: Box<[BlockState]>,
    /// Monotonic edit counter; bumps on any change to invalidate stale meshes.
    pub revision: u64,
}

impl Chunk {
    /// An empty (all-air) chunk.
    pub fn new(cx: i32, cz: i32) -> Self {
        let len = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;
        Self {
            cx,
            cz,
            blocks: vec![AIR; len].into_boxed_slice(),
            revision: 0,
        }
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        (y * CHUNK_SIZE + z) * CHUNK_SIZE + x
    }

    /// Read a block state using local coordinates
    /// (`0..CHUNK_SIZE`, `0..CHUNK_HEIGHT`, `0..CHUNK_SIZE`).
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockState {
        self.blocks[Self::index(x, y, z)]
    }

    /// Write a block state, bumping `revision` only on an actual change.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, state: BlockState) {
        let i = Self::index(x, y, z);
        if self.blocks[i] != state {
            self.blocks[i] = state;
            self.revision += 1;
        }
    }

    /// Raw contiguous block slice (index order `y, z, x`).
    pub fn blocks(&self) -> &[BlockState] {
        &self.blocks
    }
}
