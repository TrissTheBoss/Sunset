//! Collision, movement, and block raycasting against the voxel world.

use glam::Vec3;

use crate::world::World;

/// Small bias used to keep the player just off block faces.
pub const EPSILON: f32 = 0.001;

/// An axis-aligned collision box centered at a position, with half extents.
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub half: Vec3,
}

impl Aabb {
    /// Whether the box at `pos` overlaps any solid block.
    pub fn collides(&self, world: &World, pos: Vec3) -> bool {
        let min = pos - self.half;
        let max = pos + self.half;
        let min_i = min.floor();
        let max_i = max.floor();
        for y in (min_i.y as i32)..=(max_i.y as i32) {
            for z in (min_i.z as i32)..=(max_i.z as i32) {
                for x in (min_i.x as i32)..=(max_i.x as i32) {
                    if world.is_solid(x, y, z) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Move `pos` by `vel * dt`, resolving collisions one axis at a time. Returns
/// `true` if the entity ended the move standing on the ground.
pub fn move_entity(world: &World, aabb: &Aabb, pos: &mut Vec3, vel: &mut Vec3, dt: f32) -> bool {
    let mut on_ground = false;

    pos.x += vel.x * dt;
    if aabb.collides(world, *pos) {
        pos.x = resolve(world, aabb, *pos, 0, vel.x);
        vel.x = 0.0;
    }

    pos.y += vel.y * dt;
    if aabb.collides(world, *pos) {
        pos.y = resolve(world, aabb, *pos, 1, vel.y);
        if vel.y < 0.0 {
            on_ground = true;
        }
        vel.y = 0.0;
    }

    pos.z += vel.z * dt;
    if aabb.collides(world, *pos) {
        pos.z = resolve(world, aabb, *pos, 2, vel.z);
        vel.z = 0.0;
    }

    on_ground
}

/// Resolve penetration along one axis (0 = x, 1 = y, 2 = z) after moving in
/// direction `dir`. Returns the corrected coordinate along that axis.
fn resolve(world: &World, aabb: &Aabb, pos: Vec3, axis: usize, dir: f32) -> f32 {
    let min = pos - aabb.half;
    let max = pos + aabb.half;
    let mut best: Option<i32> = None;
    for y in (min.y.floor() as i32)..=(max.y.floor() as i32) {
        for z in (min.z.floor() as i32)..=(max.z.floor() as i32) {
            for x in (min.x.floor() as i32)..=(max.x.floor() as i32) {
                if !world.is_solid(x, y, z) {
                    continue;
                }
                let c = match axis {
                    0 => x,
                    1 => y,
                    _ => z,
                };
                best = Some(match best {
                    None => c,
                    Some(b) => {
                        if dir > 0.0 {
                            b.min(c)
                        } else {
                            b.max(c)
                        }
                    }
                });
            }
        }
    }
    let Some(block) = best else {
        return match axis {
            0 => pos.x,
            1 => pos.y,
            _ => pos.z,
        };
    };
    let half = match axis {
        0 => aabb.half.x,
        1 => aabb.half.y,
        _ => aabb.half.z,
    };
    if dir > 0.0 {
        block as f32 - half - EPSILON
    } else {
        block as f32 + 1.0 + half + EPSILON
    }
}

/// Whether placing a unit block at `block` would intersect the player's box at
/// `player_pos` (placement restriction, PRD 3.3 step 3).
pub fn block_intersects_player(player_pos: Vec3, block: [i32; 3]) -> bool {
    let half = Vec3::new(0.3, 0.9, 0.3);
    let pmin = player_pos - half;
    let pmax = player_pos + half;
    let bmin = Vec3::new(block[0] as f32, block[1] as f32, block[2] as f32);
    let bmax = bmin + Vec3::ONE;
    pmin.x < bmax.x
        && pmax.x > bmin.x
        && pmin.y < bmax.y
        && pmax.y > bmin.y
        && pmin.z < bmax.z
        && pmax.z > bmin.z
}

/// A voxel DDA raycast from `origin` along normalized `dir`, up to `max_dist`.
/// Returns the hit block coordinates and the face normal (pointing out of the
/// hit block toward the ray origin).
pub fn raycast(
    world: &World,
    origin: Vec3,
    dir: Vec3,
    max_dist: f32,
) -> Option<([i32; 3], [i32; 3])> {
    let mut x = origin.x.floor() as i32;
    let mut y = origin.y.floor() as i32;
    let mut z = origin.z.floor() as i32;

    let step_x: i32 = if dir.x > 0.0 { 1 } else { -1 };
    let step_y: i32 = if dir.y > 0.0 { 1 } else { -1 };
    let step_z: i32 = if dir.z > 0.0 { 1 } else { -1 };

    let t_delta_x = if dir.x != 0.0 {
        (1.0 / dir.x).abs()
    } else {
        f32::INFINITY
    };
    let t_delta_y = if dir.y != 0.0 {
        (1.0 / dir.y).abs()
    } else {
        f32::INFINITY
    };
    let t_delta_z = if dir.z != 0.0 {
        (1.0 / dir.z).abs()
    } else {
        f32::INFINITY
    };

    let mut t_max_x = if dir.x != 0.0 {
        let b = if dir.x > 0.0 {
            (x + 1) as f32
        } else {
            x as f32
        };
        (b - origin.x) / dir.x
    } else {
        f32::INFINITY
    };
    let mut t_max_y = if dir.y != 0.0 {
        let b = if dir.y > 0.0 {
            (y + 1) as f32
        } else {
            y as f32
        };
        (b - origin.y) / dir.y
    } else {
        f32::INFINITY
    };
    let mut t_max_z = if dir.z != 0.0 {
        let b = if dir.z > 0.0 {
            (z + 1) as f32
        } else {
            z as f32
        };
        (b - origin.z) / dir.z
    } else {
        f32::INFINITY
    };

    let mut face: [i32; 3] = [0, 0, 0];
    let mut t = 0.0f32;

    loop {
        if t > max_dist {
            return None;
        }
        if world.is_solid(x, y, z) {
            return Some(([x, y, z], face));
        }
        if t_max_x < t_max_y && t_max_x < t_max_z {
            x += step_x;
            t = t_max_x;
            t_max_x += t_delta_x;
            face = [-step_x, 0, 0];
        } else if t_max_y < t_max_z {
            y += step_y;
            t = t_max_y;
            t_max_y += t_delta_y;
            face = [0, -step_y, 0];
        } else {
            z += step_z;
            t = t_max_z;
            t_max_z += t_delta_z;
            face = [0, 0, -step_z];
        }
    }
}
