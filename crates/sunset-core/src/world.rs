//! The authoritative world: a sparse map of chunks addressed by signed chunk
//! coordinates. World-coordinate access uses `div_euclid`/`rem_euclid` so
//! negative coordinates resolve correctly (PRD 4.4).

use std::collections::HashMap;

use crate::block::{is_solid, BlockState, AIR};
use crate::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};

/// The authoritative block world.
#[derive(Debug, Default)]
pub struct World {
    chunks: HashMap<(i32, i32), Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow a chunk by chunk coordinates, if present.
    pub fn get_chunk(&self, cx: i32, cz: i32) -> Option<&Chunk> {
        self.chunks.get(&(cx, cz))
    }

    /// Borrow-mut a chunk, creating it if absent.
    pub fn ensure_chunk(&mut self, cx: i32, cz: i32) -> &mut Chunk {
        self.chunks
            .entry((cx, cz))
            .or_insert_with(|| Chunk::new(cx, cz))
    }

    /// Iterate over all chunks (unspecified order).
    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.values()
    }

    /// Read a block by world coordinates. Out-of-range Y or a missing chunk
    /// reads as air.
    pub fn get_block(&self, wx: i32, wy: i32, wz: i32) -> BlockState {
        if wy < 0 || wy >= CHUNK_HEIGHT as i32 {
            return AIR;
        }
        let (cx, cz) = chunk_coords(wx, wz);
        let Some(chunk) = self.chunks.get(&(cx, cz)) else {
            return AIR;
        };
        let (lx, lz) = local_coords(wx, wz);
        chunk.get(lx, wy as usize, lz)
    }

    /// Write a block by world coordinates, creating the chunk if needed.
    /// Out-of-range Y is ignored.
    pub fn set_block(&mut self, wx: i32, wy: i32, wz: i32, state: BlockState) {
        if wy < 0 || wy >= CHUNK_HEIGHT as i32 {
            return;
        }
        let (cx, cz) = chunk_coords(wx, wz);
        let (lx, lz) = local_coords(wx, wz);
        self.ensure_chunk(cx, cz).set(lx, wy as usize, lz, state);
    }

    /// Whether the block at world coordinates is solid (participates in
    /// collision). Out-of-range Y or a missing chunk is non-solid.
    pub fn is_solid(&self, wx: i32, wy: i32, wz: i32) -> bool {
        is_solid(self.get_block(wx, wy, wz))
    }
}

#[inline]
fn chunk_coords(wx: i32, wz: i32) -> (i32, i32) {
    (
        wx.div_euclid(CHUNK_SIZE as i32),
        wz.div_euclid(CHUNK_SIZE as i32),
    )
}

#[inline]
fn local_coords(wx: i32, wz: i32) -> (usize, usize) {
    (
        wx.rem_euclid(CHUNK_SIZE as i32) as usize,
        wz.rem_euclid(CHUNK_SIZE as i32) as usize,
    )
}
