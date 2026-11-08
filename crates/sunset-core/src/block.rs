//! Block registry: namespaced block states with authored development colors.
//!
//! M0 uses temporary authored materials, not target textures (PRD 3.1).
//! A block state is a compact index into [`BLOCKS`]; the durable identity is
//! the namespaced [`BlockDef::id`], never the array index (PRD 4.4).

/// A compact block-state identifier (index into [`BLOCKS`]).
pub type BlockState = u32;

/// The empty block. Air occupies no volume and is never meshed.
pub const AIR: BlockState = 0;

/// A registered block definition: a namespaced id plus an authored color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockDef {
    /// Namespaced identifier, e.g. `sunset:grass_block`.
    pub id: &'static str,
    /// Authored development color (sRGB bytes). Not a target texture.
    pub color: [u8; 3],
}

/// Authored development block set for M0. Temporary materials; not a claim of
/// vanilla parity.
pub const BLOCKS: &[BlockDef] = &[
    BlockDef { id: "sunset:air", color: [0, 0, 0] },
    BlockDef { id: "sunset:grass_block", color: [99, 165, 56] },
    BlockDef { id: "sunset:dirt", color: [134, 96, 67] },
    BlockDef { id: "sunset:stone", color: [125, 125, 125] },
    BlockDef { id: "sunset:cobblestone", color: [108, 108, 108] },
    BlockDef { id: "sunset:oak_planks", color: [162, 130, 78] },
    BlockDef { id: "sunset:sand", color: [219, 207, 163] },
];

/// Look up a block definition by state. Unknown states fall back to air.
pub fn block_def(state: BlockState) -> &'static BlockDef {
    BLOCKS.get(state as usize).unwrap_or(&BLOCKS[AIR as usize])
}

/// Look up a state by namespaced id.
pub fn state_by_id(id: &str) -> Option<BlockState> {
    BLOCKS.iter().position(|b| b.id == id).map(|i| i as BlockState)
}

/// Whether a block is treated as transparent (no face drawn against it).
/// Every M0 authored block except air is opaque.
pub fn is_transparent(state: BlockState) -> bool {
    state == AIR
}
