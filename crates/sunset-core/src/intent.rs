//! Client intents: what the player wants to do this tick. Intents cross the
//! session boundary into the authoritative simulation (PRD 4.3). Look rotation
//! is intentionally not here — it is applied client-side at render rate for
//! smoothness and is still stored authoritatively on the player.

use glam::Vec2;

/// A block-world action requested by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockAction {
    /// Remove the block at the given world coordinates.
    Break([i32; 3]),
    /// Place the selected hotbar block at the given world coordinates.
    Place([i32; 3]),
}

/// One tick of client intent.
#[derive(Debug, Clone, Default)]
pub struct PlayerIntent {
    /// Horizontal movement axis in player space: `x` = strafe (right positive),
    /// `y` = forward (positive). Produced from WASD.
    pub move_axis: Vec2,
    pub jump: bool,
    pub block_action: Option<BlockAction>,
    /// A hotbar slot the client requested to select (0-based).
    pub select_slot: Option<usize>,
}
