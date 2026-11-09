//! Versioned Sunset development save format (PRD 9.2).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::block::{block_def, state_by_id, AIR};
use crate::chunk::{CHUNK_HEIGHT, CHUNK_SIZE};
use crate::item::{item_by_id, item_id, ItemStack, EMPTY_ITEM};
use crate::player::Player;
use crate::world::World;

/// Current save schema version.
pub const SCHEMA_VERSION: u32 = 3;

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub schema_version: u32,
    pub player: PlayerSave,
    pub chunks: Vec<ChunkSave>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerSave {
    pub pos: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub selected_slot: usize,
    /// Inventory slots as (namespaced item id, count); empty slots are
    /// `minecraft:air` with count 0.
    pub inventory: Vec<ItemSave>,
    /// The 3×3 crafting grid slots.
    pub crafting: Vec<ItemSave>,
}

#[derive(Serialize, Deserialize)]
pub struct ItemSave {
    pub id: String,
    pub count: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ChunkSave {
    pub cx: i32,
    pub cz: i32,
    /// `(local x, local y, local z, namespaced block id)` for non-air blocks.
    pub blocks: Vec<(u8, u8, u8, String)>,
}

/// Serialize the current world and player to a pretty JSON string. Chunk order
/// is sorted so the output is deterministic.
pub fn to_json(world: &World, player: &Player) -> String {
    let mut chunks: Vec<ChunkSave> = world
        .chunks()
        .map(|c| {
            let mut blocks = Vec::new();
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let s = c.get(x, y, z);
                        if s != AIR {
                            blocks.push((x as u8, y as u8, z as u8, block_def(s).id.to_string()));
                        }
                    }
                }
            }
            ChunkSave {
                cx: c.cx,
                cz: c.cz,
                blocks,
            }
        })
        .collect();
    chunks.sort_by_key(|c| (c.cx, c.cz));

    let data = SaveData {
        schema_version: SCHEMA_VERSION,
        player: PlayerSave {
            pos: [player.pos.x, player.pos.y, player.pos.z],
            yaw: player.yaw,
            pitch: player.pitch,
            selected_slot: player.selected_slot,
            inventory: player
                .inventory
                .iter()
                .map(|s| ItemSave {
                    id: item_id(s.item).unwrap_or("minecraft:air").to_string(),
                    count: s.count,
                })
                .collect(),
            crafting: player
                .crafting
                .iter()
                .map(|s| ItemSave {
                    id: item_id(s.item).unwrap_or("minecraft:air").to_string(),
                    count: s.count,
                })
                .collect(),
        },
        chunks,
    };

    serde_json::to_string_pretty(&data).expect("serialize save")
}

/// Write `json` to `path` atomically (write a temp file, then rename).
pub fn write_atomic(path: &Path, json: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Load a save, reconstructing the world and player.
pub fn from_json(json: &str) -> Result<(World, Player), String> {
    let data: SaveData = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if data.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "unsupported schema version {}",
            data.schema_version
        ));
    }

    let mut world = World::new();
    for c in &data.chunks {
        for (lx, ly, lz, id) in &c.blocks {
            let s = state_by_id(id).unwrap_or(AIR);
            world.set_block(
                c.cx * CHUNK_SIZE as i32 + *lx as i32,
                *ly as i32,
                c.cz * CHUNK_SIZE as i32 + *lz as i32,
                s,
            );
        }
    }

    let p = &data.player;
    let mut player = Player::new(
        glam::Vec3::new(p.pos[0], p.pos[1], p.pos[2]),
        p.yaw,
        p.pitch,
    );
    if p.selected_slot < player.inventory.len() {
        player.selected_slot = p.selected_slot;
    }
    for (i, is) in p.inventory.iter().enumerate() {
        if i < player.inventory.len() {
            player.inventory[i] =
                ItemStack::new(item_by_id(&is.id).unwrap_or(EMPTY_ITEM), is.count);
        }
    }
    for (i, is) in p.crafting.iter().enumerate() {
        if i < player.crafting.len() {
            player.crafting[i] =
                ItemStack::new(item_by_id(&is.id).unwrap_or(EMPTY_ITEM), is.count);
        }
    }

    Ok((world, player))
}
