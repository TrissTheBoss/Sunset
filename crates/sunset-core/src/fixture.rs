//! Authored development fixtures. These are development content, not target
//! terrain generation (PRD 3.1).

use crate::block::state_by_id;
use crate::world::World;

/// The `building-yard` fixture: a small multi-chunk world centered on the
/// origin with flat ground, a wall across the origin, and raised platforms on
/// both sides of the origin. Authored development content only.
pub fn building_yard() -> World {
    let mut world = World::new();

    let grass = state_by_id("minecraft:grass_block").expect("registered block");
    let dirt = state_by_id("minecraft:dirt").expect("registered block");
    let stone = state_by_id("minecraft:stone").expect("registered block");
    let cobble = state_by_id("minecraft:cobblestone").expect("registered block");
    let planks = state_by_id("minecraft:oak_planks").expect("registered block");
    let sand = state_by_id("minecraft:sand").expect("registered block");
    // M2 blocks: non-full-cube, transparent, orientable.
    let slab_bottom =
        state_by_id("minecraft:smooth_stone_slab[type=bottom]").expect("registered block");
    let slab_top = state_by_id("minecraft:smooth_stone_slab[type=top]").expect("registered block");
    let glass = state_by_id("minecraft:glass").expect("registered block");
    let log_x = state_by_id("minecraft:oak_log[axis=x]").expect("registered block");
    let log_y = state_by_id("minecraft:oak_log[axis=y]").expect("registered block");
    let log_z = state_by_id("minecraft:oak_log[axis=z]").expect("registered block");

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

    // M2: a glass wall with a window you can see through.
    for wy in 5..=8 {
        for wx in 11..=14 {
            world.set_block(wx, wy, -12, glass);
        }
    }
    // M2: slab steps leading up to a small top-slab platform.
    for (i, wx) in (16..=20).enumerate() {
        let wy = 4 + i as i32;
        world.set_block(wx, wy, -16, slab_bottom);
    }
    world.set_block(21, 9, -16, slab_top);
    world.set_block(21, 9, -15, slab_top);
    // M2: an orientable log stack (x, then y, then z axes).
    world.set_block(-14, 5, 6, log_x);
    world.set_block(-14, 6, 6, log_y);
    world.set_block(-14, 7, 6, log_z);

    world
}
