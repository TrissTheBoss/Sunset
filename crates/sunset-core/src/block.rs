//! Block registry: namespaced block states with authored development colors.
//!
//! M1 uses the target-state candidates `minecraft:stone`, `minecraft:dirt`,
//! `minecraft:grass_block`, `minecraft:cobblestone`, and `minecraft:oak_planks`
//! (PRD 3.2). Colors are authored placeholders; visual parity is unverified.
//! The durable identity is the namespaced [`BlockDef::id`], never the array
//! index (PRD 4.4).

/// A compact block-state identifier (index into [`BLOCKS`]).
pub type BlockState = u32;

/// The empty block. Air occupies no volume and is never meshed or collided.
pub const AIR: BlockState = 0;

/// A registered block definition: a namespaced id, an authored color, and a
/// collision flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockDef {
    /// Namespaced identifier, e.g. `minecraft:grass_block`.
    pub id: &'static str,
    /// Authored development color (sRGB bytes). Not a target texture.
    pub color: [u8; 3],
    /// Whether the block has a solid collision shape. M1 content is all solid
    /// cubes; non-full or non-solid shapes arrive in later milestones.
    pub solid: bool,
}

/// Authored development block set. Temporary materials; not a claim of vanilla
/// parity.
pub const BLOCKS: &[BlockDef] = &[
    BlockDef {
        id: "minecraft:air",
        color: [0, 0, 0],
        solid: false,
    },
    BlockDef {
        id: "minecraft:stone",
        color: [125, 125, 125],
        solid: true,
    },
    BlockDef {
        id: "minecraft:dirt",
        color: [134, 96, 67],
        solid: true,
    },
    BlockDef {
        id: "minecraft:grass_block",
        color: [99, 165, 56],
        solid: true,
    },
    BlockDef {
        id: "minecraft:cobblestone",
        color: [108, 108, 108],
        solid: true,
    },
    BlockDef {
        id: "minecraft:oak_planks",
        color: [162, 130, 78],
        solid: true,
    },
    BlockDef {
        id: "sunset:sand",
        color: [219, 207, 163],
        solid: true,
    },
];

/// Look up a block definition by state. Unknown states fall back to air.
pub fn block_def(state: BlockState) -> &'static BlockDef {
    BLOCKS.get(state as usize).unwrap_or(&BLOCKS[AIR as usize])
}

/// Look up a state by namespaced id.
pub fn state_by_id(id: &str) -> Option<BlockState> {
    BLOCKS
        .iter()
        .position(|b| b.id == id)
        .map(|i| i as BlockState)
}

/// Whether a block participates in collision (solid).
pub fn is_solid(state: BlockState) -> bool {
    block_def(state).solid
}

/// Whether a block is treated as transparent for meshing (no face drawn
/// against it). Every M1 authored block except air is opaque.
pub fn is_transparent(state: BlockState) -> bool {
    state == AIR
}
