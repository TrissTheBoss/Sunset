//! Block registry: namespaced block states with separated shape, opacity, and
//! model (PRD 4.4). This separation is what lets the foundation grow: adding a
//! slab, an orientable log, or a transparent block must not rewrite the world
//! model, collision, or save identity.
//!
//! A block state is a compact index into [`BLOCKS`]; the durable identity is the
//! namespaced [`BlockDef::id`] (which encodes state properties), never the index.

use glam::Vec3;

/// A compact block-state identifier (index into [`BLOCKS`]).
pub type BlockState = u32;

/// The empty block. Air occupies no volume and is never meshed or collided.
pub const AIR: BlockState = 0;

/// A block's collision/geometry shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// No collision and no model (air).
    None,
    /// A full 1x1x1 cube.
    FullCube,
    /// A half-height slab occupying the bottom half.
    SlabBottom,
    /// A half-height slab occupying the top half.
    SlabTop,
}

/// How a block is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opacity {
    /// Fully opaque; occludes the faces of adjacent blocks.
    Opaque,
    /// Transparent; does not occlude adjacent faces and is alpha-blended.
    Transparent,
}

/// A world axis (used by orientable block models).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// A registered block definition.
#[derive(Debug, Clone, Copy)]
pub struct BlockDef {
    /// Namespaced identifier, encoding state properties, e.g.
    /// `minecraft:smooth_stone_slab[type=bottom]`.
    pub id: &'static str,
    /// Authored development color (sRGB bytes). Not a target texture.
    pub color: [u8; 3],
    pub shape: Shape,
    pub opacity: Opacity,
    /// For orientable blocks (logs): the axis whose end faces use `end_color`.
    pub end_axis: Option<Axis>,
    pub end_color: [u8; 3],
}

/// Authored development block set. Temporary materials; not a claim of vanilla
/// parity.
pub const BLOCKS: &[BlockDef] = &[
    BlockDef {
        id: "minecraft:air",
        color: [0, 0, 0],
        shape: Shape::None,
        opacity: Opacity::Transparent,
        end_axis: None,
        end_color: [0, 0, 0],
    },
    BlockDef {
        id: "minecraft:stone",
        color: [125, 125, 125],
        shape: Shape::FullCube,
        opacity: Opacity::Opaque,
        end_axis: None,
        end_color: [0, 0, 0],
    },
    BlockDef {
        id: "minecraft:dirt",
        color: [134, 96, 67],
        shape: Shape::FullCube,
        opacity: Opacity::Opaque,
        end_axis: None,
        end_color: [0, 0, 0],
    },
    BlockDef {
        id: "minecraft:grass_block",
        color: [99, 165, 56],
        shape: Shape::FullCube,
        opacity: Opacity::Opaque,
        end_axis: None,
        end_color: [0, 0, 0],
    },
    BlockDef {
        id: "minecraft:cobblestone",
        color: [108, 108, 108],
        shape: Shape::FullCube,
        opacity: Opacity::Opaque,
        end_axis: None,
        end_color: [0, 0, 0],
    },
    BlockDef {
        id: "minecraft:oak_planks",
        color: [162, 130, 78],
        shape: Shape::FullCube,
        opacity: Opacity::Opaque,
        end_axis: None,
        end_color: [0, 0, 0],
    },
    BlockDef {
        id: "minecraft:sand",
        color: [219, 207, 163],
        shape: Shape::FullCube,
        opacity: Opacity::Opaque,
        end_axis: None,
        end_color: [0, 0, 0],
    },
    // --- M2 additions: non-full-cube, transparent, orientable ---
    BlockDef {
        id: "minecraft:smooth_stone_slab[type=bottom]",
        color: [130, 130, 130],
        shape: Shape::SlabBottom,
        opacity: Opacity::Opaque,
        end_axis: None,
        end_color: [0, 0, 0],
    },
    BlockDef {
        id: "minecraft:smooth_stone_slab[type=top]",
        color: [130, 130, 130],
        shape: Shape::SlabTop,
        opacity: Opacity::Opaque,
        end_axis: None,
        end_color: [0, 0, 0],
    },
    BlockDef {
        id: "minecraft:glass",
        color: [198, 226, 226],
        shape: Shape::FullCube,
        opacity: Opacity::Transparent,
        end_axis: None,
        end_color: [0, 0, 0],
    },
    BlockDef {
        id: "minecraft:oak_log[axis=x]",
        color: [104, 78, 47],
        shape: Shape::FullCube,
        opacity: Opacity::Opaque,
        end_axis: Some(Axis::X),
        end_color: [150, 120, 80],
    },
    BlockDef {
        id: "minecraft:oak_log[axis=y]",
        color: [104, 78, 47],
        shape: Shape::FullCube,
        opacity: Opacity::Opaque,
        end_axis: Some(Axis::Y),
        end_color: [150, 120, 80],
    },
    BlockDef {
        id: "minecraft:oak_log[axis=z]",
        color: [104, 78, 47],
        shape: Shape::FullCube,
        opacity: Opacity::Opaque,
        end_axis: Some(Axis::Z),
        end_color: [150, 120, 80],
    },
];

impl BlockDef {
    /// The block's collision AABB in block-local coordinates `(min, max)`, if
    /// the block has any collision.
    pub fn collision_box(&self) -> Option<(Vec3, Vec3)> {
        match self.shape {
            Shape::None => None,
            Shape::FullCube => Some((Vec3::ZERO, Vec3::ONE)),
            Shape::SlabBottom => Some((Vec3::ZERO, Vec3::new(1.0, 0.5, 1.0))),
            Shape::SlabTop => Some((Vec3::new(0.0, 0.5, 0.0), Vec3::ONE)),
        }
    }

    /// The authored color for a face (given its outward normal), accounting for
    /// orientable blocks whose end faces use a different color.
    pub fn face_color(&self, normal: [i32; 3]) -> [u8; 3] {
        if let Some(axis) = self.end_axis {
            let is_end = match axis {
                Axis::X => normal[0] != 0,
                Axis::Y => normal[1] != 0,
                Axis::Z => normal[2] != 0,
            };
            if is_end {
                return self.end_color;
            }
        }
        self.color
    }
}

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

/// Whether a block has any collision geometry.
pub fn has_collision(state: BlockState) -> bool {
    block_def(state).collision_box().is_some()
}

/// Whether a block fully occludes the faces of adjacent blocks (i.e. an
/// adjacent face against it should not be drawn). Only full opaque cubes fully
/// occlude; partial blocks (slabs) leave the rest of the adjacent face visible.
pub fn occludes(state: BlockState) -> bool {
    let def = block_def(state);
    def.opacity == Opacity::Opaque && def.shape == Shape::FullCube
}

/// Whether a block is transparent (rendered with alpha, does not occlude).
pub fn is_transparent(state: BlockState) -> bool {
    block_def(state).opacity == Opacity::Transparent
}

/// A compact key for the occlusion information a block exposes to neighbouring
/// meshes — enough to decide whether a neighbour's faces change. Material
/// (color/texture) is deliberately excluded so a material-only change (e.g.
/// dirt → stone) does not invalidate neighbours (M14).
pub fn occlusion_key(state: BlockState) -> u8 {
    let def = block_def(state);
    match def.shape {
        Shape::None => 0,
        Shape::FullCube => {
            if def.opacity == Opacity::Opaque {
                1
            } else {
                2
            }
        }
        Shape::SlabBottom => 3,
        Shape::SlabTop => 4,
    }
}
