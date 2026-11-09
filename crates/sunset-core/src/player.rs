//! Authoritative player state: position, velocity, rotation, and inventory.
//!
//! Position is the feet (bottom-center of the collision box). Rotation is stored
//! here so it persists in saves; look input mutates it directly at render rate
//! while movement goes through the fixed-tick simulation.

use glam::Vec3;

use crate::item::{item_by_id, item_def, Item, ItemStack};

/// Player half-width (X/Z extent).
pub const PLAYER_HALF_WIDTH: f32 = 0.3;
/// Player height (Y extent).
pub const PLAYER_HEIGHT: f32 = 1.8;
/// Eye height above the feet.
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;
/// Hotbar slots (the first slots of the inventory).
pub const HOTBAR_SIZE: usize = 9;
/// Main-inventory slots (after the hotbar).
pub const MAIN_SIZE: usize = 18;
/// Total inventory slots.
pub const INVENTORY_SIZE: usize = HOTBAR_SIZE + MAIN_SIZE;
/// Maximum player health (20 = 10 hearts).
pub const MAX_HEALTH: f32 = 20.0;
/// Fall distance (in blocks) before fall damage begins.
pub const FALL_DAMAGE_THRESHOLD: f32 = 3.0;

/// Authoritative player state.
#[derive(Debug, Clone)]
pub struct Player {
    pub pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    /// Current health (0..=MAX_HEALTH).
    pub health: f32,
    /// Distance fallen since leaving the ground, for fall damage.
    pub fall_distance: f32,
    /// Where the player respawns after death.
    pub spawn_pos: Vec3,
    /// The inventory; the first [`HOTBAR_SIZE`] slots are the hotbar.
    pub inventory: [ItemStack; INVENTORY_SIZE],
    pub selected_slot: usize,
    /// The 3×3 crafting grid (flattened row-major).
    pub crafting: [ItemStack; 9],
}

impl Player {
    pub fn new(pos: Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            pos,
            vel: Vec3::ZERO,
            yaw,
            pitch,
            on_ground: false,
            health: MAX_HEALTH,
            fall_distance: 0.0,
            spawn_pos: pos,
            inventory: default_inventory(),
            selected_slot: 0,
            crafting: [ItemStack::EMPTY; 9],
        }
    }

    /// Eye (camera) position in world space. `pos` is the center of the
    /// collision box, so the eye sits at `eye_height` above the feet.
    pub fn eye_pos(&self) -> Vec3 {
        self.pos + Vec3::new(0.0, PLAYER_EYE_HEIGHT - PLAYER_HEIGHT * 0.5, 0.0)
    }

    /// Add `count` of `item` to the inventory, stacking onto existing stacks
    /// (hotbar first) then filling empty slots. Returns the count that did not
    /// fit (0 when everything was added).
    pub fn add_item(&mut self, item: Item, count: u32) -> u32 {
        let max = item_def(item).map(|d| d.max_stack).unwrap_or(0);
        let mut remaining = count;
        for slot in &mut self.inventory {
            if remaining == 0 {
                break;
            }
            if slot.item == item && slot.count < max {
                let add = (max - slot.count).min(remaining);
                slot.count += add;
                remaining -= add;
            }
        }
        for slot in &mut self.inventory {
            if remaining == 0 {
                break;
            }
            if slot.is_empty() {
                let add = max.min(remaining);
                *slot = ItemStack::new(item, add);
                remaining -= add;
            }
        }
        remaining
    }

    /// Whether the selected hotbar item is a pickaxe.
    pub fn holding_pickaxe(&self) -> bool {
        crate::item::is_pickaxe(self.inventory[self.selected_slot].item)
    }

    /// Craft using the 3×3 crafting grid, consuming one of each input and adding
    /// the output to the inventory. Returns the output (item, count), or `None`
    /// if no recipe matches.
    pub fn craft(&mut self) -> Option<(Item, u32)> {
        let out = crate::recipe::craft(&self.crafting)?;
        for slot in &mut self.crafting {
            if !slot.is_empty() {
                slot.count -= 1;
                if slot.count == 0 {
                    *slot = ItemStack::EMPTY;
                }
            }
        }
        self.add_item(out.0, out.1);
        Some(out)
    }
}

/// The starting inventory: the target blocks in the hotbar (count 1 each) and an
/// empty main inventory. The player obtains more materials by breaking blocks.
pub fn default_inventory() -> [ItemStack; INVENTORY_SIZE] {
    let mut inv = [ItemStack::EMPTY; INVENTORY_SIZE];
    for (i, id) in [
        "minecraft:stone",
        "minecraft:dirt",
        "minecraft:grass_block",
        "minecraft:cobblestone",
        "minecraft:oak_planks",
        "minecraft:smooth_stone_slab[type=bottom]",
        "minecraft:glass",
    ]
    .iter()
    .enumerate()
    {
        inv[i] = ItemStack::new(item_by_id(id).expect("registered item"), 1);
    }
    inv
}

