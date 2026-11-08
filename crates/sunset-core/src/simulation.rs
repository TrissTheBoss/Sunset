//! The local authoritative simulation: applies client intents to the world and
//! player at a fixed tick, independent of rendering (PRD 4.3, 4.5).

use crate::block::AIR;
use crate::intent::{BlockAction, PlayerIntent};
use crate::physics::{self, Aabb};
use crate::player::{Player, PLAYER_HALF_WIDTH, PLAYER_HEIGHT};
use crate::world::World;

/// Downward acceleration (blocks/s^2).
pub const GRAVITY: f32 = 32.0;
/// Horizontal walk speed (blocks/s).
pub const WALK_SPEED: f32 = 5.6;
/// Upward velocity applied on jump (blocks/s). 9.8 yields ~1.25 blocks of
/// discrete jump height, enough to climb a one-block ledge.
pub const JUMP_VELOCITY: f32 = 9.8;

/// The authoritative game state: a world and a player, stepped at a fixed tick.
pub struct Simulation {
    pub world: World,
    pub player: Player,
    /// Position before the last step, used for render interpolation.
    pub prev_pos: glam::Vec3,
}

impl Simulation {
    pub fn new(world: World, player: Player) -> Self {
        let prev_pos = player.pos;
        Self {
            world,
            player,
            prev_pos,
        }
    }

    /// Apply one tick of client intent over fixed timestep `dt`.
    pub fn step(&mut self, intent: &PlayerIntent, dt: f32) {
        self.prev_pos = self.player.pos;

        // Horizontal wish direction from yaw (ignore pitch).
        let yaw_quat = glam::Quat::from_rotation_y(self.player.yaw);
        let forward = yaw_quat * glam::Vec3::NEG_Z;
        let right = yaw_quat * glam::Vec3::X;
        let wish = forward * intent.move_axis.y + right * intent.move_axis.x;

        // Horizontal velocity (instant, Creative-style movement).
        self.player.vel.x = wish.x * WALK_SPEED;
        self.player.vel.z = wish.z * WALK_SPEED;

        // Jump.
        if intent.jump && self.player.on_ground {
            self.player.vel.y = JUMP_VELOCITY;
            self.player.on_ground = false;
        }

        // Gravity.
        self.player.vel.y -= GRAVITY * dt;

        // Move with collision.
        let aabb = Aabb {
            half: glam::Vec3::new(PLAYER_HALF_WIDTH, PLAYER_HEIGHT * 0.5, PLAYER_HALF_WIDTH),
        };
        self.player.on_ground = physics::move_entity(
            &self.world,
            &aabb,
            &mut self.player.pos,
            &mut self.player.vel,
            dt,
        );

        // Hotbar selection.
        if let Some(slot) = intent.select_slot {
            if slot < self.player.hotbar.len() {
                self.player.selected_slot = slot;
            }
        }

        // Block edits.
        if let Some(action) = intent.block_action {
            match action {
                BlockAction::Break(pos) => {
                    self.world.set_block(pos[0], pos[1], pos[2], AIR);
                }
                BlockAction::Place(pos) => {
                    let block = self.player.hotbar[self.player.selected_slot];
                    if block == AIR {
                        return;
                    }
                    if physics::block_intersects_player(self.player.pos, pos, block) {
                        return;
                    }
                    self.world.set_block(pos[0], pos[1], pos[2], block);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use crate::intent::{BlockAction, PlayerIntent};
    use crate::save;

    const DT: f32 = 1.0 / 20.0;

    /// Replay a fixed authoritative interaction sequence headlessly and return
    /// the deterministic serialized end state. This is the M2 renderer-free
    /// replay of the supported M1 interaction sequence (PRD 5.1).
    fn replay() -> String {
        let world = fixture::building_yard();
        let player = Player::new(glam::Vec3::new(8.0, 5.5, 16.0), 0.5, -0.25);
        let mut sim = Simulation::new(world, player);

        let seq = [
            PlayerIntent {
                move_axis: glam::Vec2::new(0.0, 1.0),
                ..Default::default()
            },
            PlayerIntent {
                move_axis: glam::Vec2::new(0.0, 1.0),
                ..Default::default()
            },
            PlayerIntent {
                move_axis: glam::Vec2::new(0.0, 1.0),
                jump: true,
                ..Default::default()
            },
            PlayerIntent {
                block_action: Some(BlockAction::Break([9, 5, 16])),
                ..Default::default()
            },
            PlayerIntent {
                block_action: Some(BlockAction::Place([9, 5, 16])),
                ..Default::default()
            },
        ];

        for intent in &seq {
            sim.step(intent, DT);
        }

        save::to_json(&sim.world, &sim.player)
    }

    #[test]
    fn replay_is_deterministic() {
        assert_eq!(replay(), replay());
    }

    #[test]
    fn replay_roundtrips_through_save() {
        let json = replay();
        let (world, player) = save::from_json(&json).expect("load save");
        assert_eq!(save::to_json(&world, &player), json);
    }

    #[test]
    fn player_settles_on_ground() {
        let world = fixture::building_yard();
        let player = Player::new(glam::Vec3::new(8.0, 5.5, 16.0), 0.5, -0.25);
        let mut sim = Simulation::new(world, player);
        for _ in 0..(20 * 3) {
            sim.step(&PlayerIntent::default(), DT);
        }
        let p = sim.player.pos;
        assert!(
            (p.x - 8.0).abs() < 0.1 && (p.z - 16.0).abs() < 0.1,
            "player drifted horizontally: {p:?}"
        );
        // `pos` is the collision-box center (feet at y=5.0, center at ~5.9).
        assert!((p.y - 5.9).abs() < 0.5, "player not on ground: {p:?}");
    }

    #[test]
    fn jump_clears_one_block() {
        let world = fixture::building_yard();
        let player = Player::new(glam::Vec3::new(8.0, 5.5, 16.0), 0.5, -0.25);
        let mut sim = Simulation::new(world, player);
        // Settle onto the ground first.
        for _ in 0..(20 * 3) {
            sim.step(&PlayerIntent::default(), DT);
        }
        let ground_center = sim.player.pos.y;
        // Jump once, then track the peak of the box center.
        sim.step(
            &PlayerIntent {
                jump: true,
                ..Default::default()
            },
            DT,
        );
        let mut peak = sim.player.pos.y;
        for _ in 0..(20 * 2) {
            sim.step(&PlayerIntent::default(), DT);
            peak = peak.max(sim.player.pos.y);
        }
        // Feet must rise more than one block above the ground.
        let feet_rise = peak - ground_center;
        assert!(feet_rise > 1.0, "jump too short: {feet_rise}");
    }
}
