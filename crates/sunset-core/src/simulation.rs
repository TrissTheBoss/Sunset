//! The local authoritative simulation: applies client intents to the world and
//! player at a fixed tick, independent of rendering (PRD 4.3, 4.5).

use crate::block::AIR;
use crate::intent::{BlockAction, PlayerIntent};
use crate::item::{block_drop, item_def, ItemStack};
use crate::physics::{self, Aabb};
use crate::player::{Player, HOTBAR_SIZE, PLAYER_HALF_WIDTH, PLAYER_HEIGHT};
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
            if slot < HOTBAR_SIZE {
                self.player.selected_slot = slot;
            }
        }

        // Block edits.
        if let Some(action) = intent.block_action {
            match action {
                BlockAction::Break(pos) => {
                    let old = self.world.get_block(pos[0], pos[1], pos[2]);
                    if old == AIR {
                        return;
                    }
                    self.world.set_block(pos[0], pos[1], pos[2], AIR);
                    // Drop the block's item (respecting tool rules): stone
                    // needs a pickaxe and drops cobblestone.
                    let drop = block_drop(old);
                    if !drop.needs_pickaxe || self.player.holding_pickaxe() {
                        self.player.add_item(drop.item, 1);
                    }
                }
                BlockAction::Place(pos) => {
                    let slot = self.player.inventory[self.player.selected_slot];
                    let Some(block) = item_def(slot.item).and_then(|d| d.block) else {
                        return;
                    };
                    if block == AIR {
                        return;
                    }
                    if physics::block_intersects_player(self.player.pos, pos, block) {
                        return;
                    }
                    self.world.set_block(pos[0], pos[1], pos[2], block);
                    // Consume one item from the selected slot.
                    let slot = &mut self.player.inventory[self.player.selected_slot];
                    slot.count -= 1;
                    if slot.count == 0 {
                        *slot = ItemStack::EMPTY;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::state_by_id;
    use crate::fixture;
    use crate::intent::{BlockAction, PlayerIntent};
    use crate::item::item_by_id;
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

    #[test]
    fn breaking_a_block_drops_its_item() {
        let mut world = World::new();
        let dirt = state_by_id("minecraft:dirt").unwrap();
        world.set_block(0, 0, 0, dirt);
        let mut sim = Simulation::new(
            world,
            Player::new(glam::Vec3::new(5.0, 5.0, 5.0), 0.0, 0.0),
        );
        let dirt_item = item_by_id("minecraft:dirt").unwrap();
        let before: u32 = sim
            .player
            .inventory
            .iter()
            .filter(|s| s.item == dirt_item)
            .map(|s| s.count)
            .sum();
        sim.step(
            &PlayerIntent {
                block_action: Some(BlockAction::Break([0, 0, 0])),
                ..Default::default()
            },
            DT,
        );
        let after: u32 = sim
            .player
            .inventory
            .iter()
            .filter(|s| s.item == dirt_item)
            .map(|s| s.count)
            .sum();
        assert_eq!(after, before + 1, "breaking dirt should add one dirt item");
        assert_eq!(sim.world.get_block(0, 0, 0), AIR);
    }

    #[test]
    fn stone_requires_pickaxe_to_drop_cobblestone() {
        let cobble_item = item_by_id("minecraft:cobblestone").unwrap();
        let wooden_pickaxe = item_by_id("minecraft:wooden_pickaxe").unwrap();
        let stone = state_by_id("minecraft:stone").unwrap();

        let mut world = World::new();
        world.set_block(0, 0, 0, stone);
        let mut sim = Simulation::new(
            world,
            Player::new(glam::Vec3::new(5.0, 5.0, 5.0), 0.0, 0.0),
        );
        let count = |p: &Player| {
            p.inventory
                .iter()
                .filter(|s| s.item == cobble_item)
                .map(|s| s.count)
                .sum::<u32>()
        };

        // Bare hands: stone breaks but drops nothing.
        let before = count(&sim.player);
        sim.step(
            &PlayerIntent {
                block_action: Some(BlockAction::Break([0, 0, 0])),
                ..Default::default()
            },
            DT,
        );
        assert_eq!(sim.world.get_block(0, 0, 0), AIR);
        assert_eq!(count(&sim.player), before, "stone without a pickaxe drops nothing");

        // With a pickaxe: stone drops cobblestone.
        sim.world.set_block(0, 0, 0, stone);
        sim.player.inventory[0] = ItemStack::new(wooden_pickaxe, 1);
        sim.player.selected_slot = 0;
        let before = count(&sim.player);
        sim.step(
            &PlayerIntent {
                block_action: Some(BlockAction::Break([0, 0, 0])),
                ..Default::default()
            },
            DT,
        );
        assert_eq!(count(&sim.player), before + 1, "stone with a pickaxe drops cobblestone");
    }

    #[test]
    fn placing_a_block_consumes_an_item() {
        let world = fixture::building_yard();
        let mut sim = Simulation::new(
            world,
            Player::new(glam::Vec3::new(8.0, 5.5, 16.0), 0.5, -0.25),
        );
        // Slot 0 is stone (from the default inventory).
        sim.player.selected_slot = 0;
        let before = sim.player.inventory[0].count;
        assert!(before > 0, "slot 0 should start non-empty");
        sim.step(
            &PlayerIntent {
                block_action: Some(BlockAction::Place([8, 20, 16])),
                ..Default::default()
            },
            DT,
        );
        assert_eq!(
            sim.player.inventory[0].count,
            before - 1,
            "placing should consume one item"
        );
        assert_eq!(
            sim.world.get_block(8, 20, 16),
            state_by_id("minecraft:stone").unwrap()
        );
    }
}
