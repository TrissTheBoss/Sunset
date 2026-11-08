//! Authored development fixtures. These are development content, not target
//! terrain generation (PRD 3.1).

use crate::block::state_by_id;
use crate::world::World;

/// The `building-yard` fixture: a small multi-chunk world centered on the
/// origin with flat ground, a wall across the origin, and raised platforms on
/// both sides of the origin. Authored development content only.
pub fn building_yard() -> World {
    let mut world = World::new();

    let grass = state_by_id("sunset:grass_block").expect("registered block");
    let dirt = state_by_id("sunset:dirt").expect("registered block");
    let stone = state_by_id("sunset:stone").expect("registered block");
    let cobble = state_by_id("sunset:cobblestone").expect("registered block");
    let planks = state_by_id("sunset:oak_planks").expect("registered block");
    let sand = state_by_id("sunset:sand").expect("registered block");

    // Flat ground spanning several chunks on both sides of the origin.
    for wx in -32..=32 {
        for wz in -32..=32 {
            world.set_block(wx, 0, wz, stone);
            world.set_block(wx, 1, wz, stone);
            world.set_block(wx, 2, wz, dirt);
            world.set_block(wx, 3, wz, dirt);
            world.set_block(wx, 4, wz, grass);
        }
    }

    // A cobblestone wall crossing the origin (a chunk boundary).
    for wy in 5..=9 {
        for wz in -8..=8 {
            world.set_block(0, wy, wz, cobble);
        }
    }

    // Raised stone platform (positive quadrant).
    for wx in 3..=8 {
        for wz in 3..=8 {
            world.set_block(wx, 5, wz, stone);
        }
    }

    // Taller oak-plank platform (negative quadrant).
    for wx in -9..=-4 {
        for wz in -9..=-4 {
            world.set_block(wx, 5, wz, planks);
            world.set_block(wx, 6, wz, planks);
        }
    }

    // A few scattered sand blocks for visual variety.
    for (wx, wz) in [(-16, 10), (12, -14), (18, 6), (-20, -4)] {
        world.set_block(wx, 5, wz, sand);
    }

    world
}
