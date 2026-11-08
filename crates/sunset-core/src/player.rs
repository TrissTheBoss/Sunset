//! Authoritative player state: position, velocity, rotation, and hotbar.
//!
//! Position is the player's feet (bottom-center of the collision box). Rotation
//! is stored here so it persists in saves; look input mutates it directly at
//! render rate while movement goes through the fixed-tick simulation.

use glam::Vec3;

use crate::block::{state_by_id, BlockState, AIR};

/// Player half-width (X/Z extent).
pub const PLAYER_HALF_WIDTH: f32 = 0.3;
/// Player height (Y extent).
pub const PLAYER_HEIGHT: f32 = 1.8;
/// Eye height above the feet.
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;

/// Authoritative player state.
#[derive(Debug, Clone)]
pub struct Player {
    pub pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    /// Nine-slot hotbar; empty slots are [`AIR`].
    pub hotbar: [BlockState; 9],
    pub selected_slot: usize,
}

impl Player {
    pub fn new(pos: Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            pos,
            vel: Vec3::ZERO,
            yaw,
            pitch,
            on_ground: false,
            hotbar: default_hotbar(),
            selected_slot: 0,
        }
    }

    /// Eye (camera) position in world space.
    pub fn eye_pos(&self) -> Vec3 {
        self.pos + Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0)
    }
}

/// The hotbar: the five supported target blocks plus the M2 slab and glass,
/// then empty slots.
pub fn default_hotbar() -> [BlockState; 9] {
    [
        state_by_id("minecraft:stone").unwrap(),
        state_by_id("minecraft:dirt").unwrap(),
        state_by_id("minecraft:grass_block").unwrap(),
        state_by_id("minecraft:cobblestone").unwrap(),
        state_by_id("minecraft:oak_planks").unwrap(),
        state_by_id("minecraft:smooth_stone_slab[type=bottom]").unwrap(),
        state_by_id("minecraft:glass").unwrap(),
        AIR,
        AIR,
    ]
}
