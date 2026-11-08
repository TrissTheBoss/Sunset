//! Loads Minecraft block textures from a local, user-provided Minecraft
//! installation (PRD: local import of user assets; M2 target-resources probe).
//!
//! Block models and blockstates live inside the client jar
//! (`versions/<version>/<version>.jar`), not the asset index. This module reads
//! the jar, resolves each supported block's face textures, and packs them into
//! a texture atlas.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use sunset_core::block::{Axis, BlockState};

// --- Minecraft JSON types (subset of the model format) ---

/// The texture names assigned to a block's faces.
#[derive(Default, Clone)]
pub struct FaceTextures {
    pub all: Option<String>,
    pub top: Option<String>,
    pub bottom: Option<String>,
    pub side: Option<String>,
    pub end: Option<String>,
}

impl FaceTextures {
    /// The texture for a face with the given outward normal, accounting for an
    /// orientable block's end axis.
    pub fn texture_for_face(&self, normal: [i32; 3], end_axis: Option<Axis>) -> Option<&str> {
        if let Some(all) = &self.all {
            return Some(all);
        }
        if let Some(axis) = end_axis {
            let is_end = match axis {
                Axis::X => normal[0] != 0,
                Axis::Y => normal[1] != 0,
                Axis::Z => normal[2] != 0,
            };
            if is_end {
                return self.end.as_deref();
            }
            return self.side.as_deref();
        }
        if normal[1] > 0 {
            self.top.as_deref()
        } else if normal[1] < 0 {
            self.bottom.as_deref()
        } else {
            self.side.as_deref()
        }
    }
}

/// The loaded assets: per-block face textures plus a texture atlas.
pub struct LoadedAssets {
    pub face_textures: HashMap<BlockState, FaceTextures>,
    /// Texture name → atlas UV rect `(u0, v0, u1, v1)`.
    pub uv: HashMap<String, [f32; 4]>,
    /// The packed atlas image (RGBA).
    pub atlas: image::RgbaImage,
}

/// Maps a supported Sunset block id to its Minecraft block name and variant
/// key. Returns `None` for development-only blocks (no target texture).
fn mc_block_ref(id: &str) -> Option<(&'static str, &'static str)> {
    Some(match id {
        "minecraft:stone" => ("stone", ""),
        "minecraft:dirt" => ("dirt", ""),
        "minecraft:grass_block" => ("grass_block", "snowy=false"),
        "minecraft:cobblestone" => ("cobblestone", ""),
        "minecraft:oak_planks" => ("oak_planks", ""),
        "minecraft:sand" => ("sand", ""),
        "minecraft:glass" => ("glass", ""),
        "minecraft:smooth_stone_slab[type=bottom]" => ("smooth_stone_slab", "type=bottom"),
        "minecraft:smooth_stone_slab[type=top]" => ("smooth_stone_slab", "type=top"),
        "minecraft:oak_log[axis=x]" => ("oak_log", "axis=x"),
        "minecraft:oak_log[axis=y]" => ("oak_log", "axis=y"),
        "minecraft:oak_log[axis=z]" => ("oak_log", "axis=z"),
        _ => return None,
    })
}

/// Locate the `.minecraft` directory (default install or `SUNSET_MINECRAFT_DIR`).
pub fn find_minecraft_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SUNSET_MINECRAFT_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = Path::new(&appdata).join(".minecraft");
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

/// Locate the client jar (preferring an exact `26.3` folder).
fn find_jar(minecraft_dir: &Path) -> Option<PathBuf> {
    let versions = minecraft_dir.join("versions");
    let jar = versions.join("26.3").join("26.3.jar");
    if jar.is_file() {
        return Some(jar);
    }
    if let Ok(entries) = std::fs::read_dir(&versions) {
        for entry in entries.flatten() {
            let dir = entry.path();
            if dir.is_dir() {
                let name = dir.file_name()?.to_string_lossy().to_string();
                let candidate = dir.join(format!("{name}.jar"));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

/// Read a whole entry from an already-open jar.
fn read_jar_entry(archive: &mut zip::ZipArchive<File>, entry: &str) -> Result<Vec<u8>, String> {
    let mut f = archive
        .by_name(entry)
        .map_err(|e| format!("{entry}: {e}"))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

/// Resolve a texture reference (e.g. `minecraft:block/stone`) to a texture name.
fn texture_name(reference: &str) -> String {
    reference
        .strip_prefix("minecraft:")
        .unwrap_or(reference)
        .to_string()
}

/// Resolve a texture name (e.g. `block/stone`) to a jar entry path.
fn texture_entry(name: &str) -> String {
    format!("assets/minecraft/textures/{name}.png")
}

/// Load the supported blocks' face textures and build a texture atlas from the
/// user's local Minecraft installation.
pub fn load_assets(minecraft_dir: &Path) -> Result<LoadedAssets, String> {
    let jar = find_jar(minecraft_dir).ok_or("no client jar found in versions/")?;
    let file = File::open(&jar).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut face_textures: HashMap<BlockState, FaceTextures> = HashMap::new();
    let mut texture_bytes: HashMap<String, Vec<u8>> = HashMap::new();

    for (state, def) in sunset_core::BLOCKS.iter().enumerate() {
        if state == 0 {
            continue; // air
        }
        let Some((block, variant)) = mc_block_ref(def.id) else {
            continue;
        };

        let state_entry = format!("assets/minecraft/blockstates/{block}.json");
        let state_bytes = read_jar_entry(&mut archive, &state_entry)?;
        let state_json: serde_json::Value =
            serde_json::from_slice(&state_bytes).map_err(|e| format!("{state_entry}: {e}"))?;
        let variant_value = &state_json["variants"][variant];
        let model_ref = match variant_value {
            serde_json::Value::Object(obj) => obj["model"].as_str(),
            serde_json::Value::Array(arr) => arr.first().and_then(|v| v["model"].as_str()),
            _ => None,
        }
        .ok_or_else(|| format!("no variant '{variant}' in {block}"))?
        .to_string();

        let model_path = model_ref.strip_prefix("minecraft:").unwrap_or(&model_ref);
        let model_entry = format!("assets/minecraft/models/{model_path}.json");
        let model_bytes = read_jar_entry(&mut archive, &model_entry)?;
        let model: serde_json::Value =
            serde_json::from_slice(&model_bytes).map_err(|e| format!("{model_entry}: {e}"))?;

        let mut ft = FaceTextures::default();
        if let Some(textures) = model["textures"].as_object() {
            for (key, value) in textures {
                // A texture value is a string or an object with a `sprite` field.
                let name = match value {
                    serde_json::Value::String(s) => Some(texture_name(s)),
                    serde_json::Value::Object(obj) => {
                        obj.get("sprite").and_then(|s| s.as_str()).map(texture_name)
                    }
                    _ => None,
                };
                let Some(name) = name else {
                    continue;
                };
                match key.as_str() {
                    "all" => ft.all = Some(name.clone()),
                    "top" => ft.top = Some(name.clone()),
                    "bottom" => ft.bottom = Some(name.clone()),
                    "side" => ft.side = Some(name.clone()),
                    "end" => ft.end = Some(name.clone()),
                    _ => {}
                }
                if let std::collections::hash_map::Entry::Vacant(e) =
                    texture_bytes.entry(name.clone())
                {
                    if let Ok(bytes) = read_jar_entry(&mut archive, &texture_entry(&name)) {
                        e.insert(bytes);
                    }
                }
            }
        }

        face_textures.insert(state as BlockState, ft);
    }

    // Build the atlas.
    let mut names: Vec<&String> = texture_bytes.keys().collect();
    names.sort();
    let n = names.len();
    if n == 0 {
        return Err("no textures resolved".into());
    }
    let grid = (n as f32).sqrt().ceil() as u32;
    let cell = 16u32;
    let size = grid * cell;
    let mut atlas = image::RgbaImage::new(size, size);
    let mut uv = HashMap::new();
    for (i, name) in names.iter().enumerate() {
        let bytes = &texture_bytes[*name];
        let img = image::load_from_memory(bytes)
            .map_err(|e| format!("decode {name}: {e}"))?
            .to_rgba8();
        let img = if img.width() == cell && img.height() == cell {
            img
        } else {
            image::imageops::resize(&img, cell, cell, image::imageops::FilterType::Nearest)
        };
        let col = (i as u32) % grid;
        let row = (i as u32) / grid;
        let (x, y) = (col * cell, row * cell);
        image::imageops::replace(&mut atlas, &img, x as i64, y as i64);
        let u0 = x as f32 / size as f32;
        let v0 = y as f32 / size as f32;
        let u1 = (x + cell) as f32 / size as f32;
        let v1 = (y + cell) as f32 / size as f32;
        uv.insert((*name).clone(), [u0, v0, u1, v1]);
    }

    Ok(LoadedAssets {
        face_textures,
        uv,
        atlas,
    })
}
