//! Loads Minecraft block textures from a local, user-provided Minecraft
//! installation (PRD: local import of user assets).

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use sunset_core::block::{Axis, BlockState};
use sunset_core::item::{item_defs, Item};

/// The texture names assigned to a block's faces.
#[derive(Default, Clone)]
pub struct FaceTextures {
    pub all: Option<String>,
    pub top: Option<String>,
    pub bottom: Option<String>,
    pub side: Option<String>,
    pub end: Option<String>,
    pub front: Option<String>,
    pub particle: Option<String>,
    /// Directional cube keys (used by `cube`-parented models like the crafting
    /// table, which name faces `up`/`north`/`east`/… instead of `top`/`side`).
    pub up: Option<String>,
    pub north: Option<String>,
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
    /// The UI atlas (item icons, HUD sprites, container backgrounds, font).
    pub ui_atlas: image::RgbaImage,
    /// Named sprite/container/font region → UV rect in `ui_atlas`.
    pub ui_uvs: HashMap<String, [f32; 4]>,
    /// Item id → item icon UV rect in `ui_atlas`.
    pub item_icons: HashMap<sunset_core::Item, [f32; 4]>,
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
        "minecraft:crafting_table" => ("crafting_table", ""),
        "minecraft:furnace" => ("furnace", "facing=north,lit=false"),
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
/// user's local Minecraft installation, plus any runtime-registered custom UI
/// textures in `extra`.
pub fn load_assets(
    minecraft_dir: &Path,
    extra: &HashMap<String, image::RgbaImage>,
) -> Result<LoadedAssets, String> {
    let jar = find_jar(minecraft_dir).ok_or("no client jar found in versions/")?;
    let file = File::open(&jar).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut face_textures: HashMap<BlockState, FaceTextures> = HashMap::new();
    let mut texture_bytes: HashMap<String, Vec<u8>> = HashMap::new();

    for (state, def) in sunset_core::block_defs() {
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
                    "front" => ft.front = Some(name.clone()),
                    "particle" => ft.particle = Some(name.clone()),
                    "up" => ft.up = Some(name.clone()),
                    "north" => ft.north = Some(name.clone()),
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

        face_textures.insert(state, ft);
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

    // Build the UI atlas: item icons + HUD/container sprites + font.
    let (ui_atlas, ui_uvs, item_icons) = build_ui_atlas(&mut archive, &face_textures, extra)?;

    Ok(LoadedAssets {
        face_textures,
        uv,
        atlas,
        ui_atlas,
        ui_uvs,
        item_icons,
    })
}

/// Decode a texture image by its name (e.g. `"block/stone"`), resized to 16×16.
fn load_texture_image(
    archive: &mut zip::ZipArchive<File>,
    name: &str,
) -> Option<image::RgbaImage> {
    let bytes = read_jar_entry(archive, &texture_entry(name)).ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    Some(ensure_16(&img))
}

/// Resize an image to 16×16 (nearest) if it is not already.
fn ensure_16(img: &image::RgbaImage) -> image::RgbaImage {
    if img.width() == 16 && img.height() == 16 {
        img.clone()
    } else {
        image::imageops::resize(img, 16, 16, image::imageops::FilterType::Nearest)
    }
}

/// Build an isometric "3D" block-item sprite from the block's top and side
/// textures (a cube seen from an elevated 45° angle, like Minecraft's block
/// items). The top face is unshaded, the front face is slightly shaded, and the
/// side face is darker, so the sprite reads as three-dimensional.
fn isometric_block_icon(
    top: &image::RgbaImage,
    side: &image::RgbaImage,
    half_height: bool,
) -> image::RgbaImage {
    let mut out = image::RgbaImage::new(16, 16);
    // Shift the cube by -0.5 so its edges land on pixel boundaries instead of
    // being clipped 1px on the right/bottom.
    // Top face (diamond): tex (0,0)->(7.5,-0.5), (1,0)->(15.5,3.5), (0,1)->(-0.5,3.5).
    map_face(&mut out, (7.5, -0.5), (15.5, 3.5), (-0.5, 3.5), top, 1.0);
    // Front face (left parallelogram), full or half height.
    let front_bottom = if half_height { 7.5 } else { 11.5 };
    map_face(&mut out, (-0.5, 3.5), (7.5, 7.5), (-0.5, front_bottom), side, 0.8);
    // Side face (right parallelogram), full or half height.
    let side_bottom = if half_height { 11.5 } else { 15.5 };
    map_face(&mut out, (7.5, 7.5), (15.5, 3.5), (7.5, side_bottom), side, 0.6);
    out
}

/// Build a block item's isometric sprite from its resolved top + side textures.
fn block_item_isometric(
    archive: &mut zip::ZipArchive<File>,
    ft: &FaceTextures,
    half_height: bool,
) -> Option<image::RgbaImage> {
    let top_name = ft
        .top
        .clone()
        .or_else(|| ft.up.clone())
        .or_else(|| ft.end.clone())
        .or_else(|| ft.all.clone())?;
    let side_name = ft
        .side
        .clone()
        .or_else(|| ft.north.clone())
        .or_else(|| ft.all.clone())?;
    let top = load_texture_image(archive, &top_name)?;
    let side = load_texture_image(archive, &side_name)?;
    Some(isometric_block_icon(&top, &side, half_height))
}

/// Map a texture onto a parallelogram (defined by its three corners for tex
/// (0,0), (1,0) and (0,1)) via an inverse affine transform, shading each texel
/// by `shade`.
fn map_face(
    out: &mut image::RgbaImage,
    p00: (f32, f32),
    p10: (f32, f32),
    p01: (f32, f32),
    tex: &image::RgbaImage,
    shade: f32,
) {
    let e0 = (p10.0 - p00.0, p10.1 - p00.1);
    let e1 = (p01.0 - p00.0, p01.1 - p00.1);
    let det = e0.0 * e1.1 - e0.1 * e1.0;
    if det.abs() < 1e-6 {
        return;
    }
    let tw = tex.width() as f32;
    let th = tex.height() as f32;
    for y in 0..out.height() {
        for x in 0..out.width() {
            let dx = x as f32 + 0.5 - p00.0;
            let dy = y as f32 + 0.5 - p00.1;
            let u = (dx * e1.1 - dy * e1.0) / det;
            let v = (dy * e0.0 - dx * e0.1) / det;
            if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 {
                continue;
            }
            let tx = (u * (tw - 1.0)).round() as u32;
            let ty = (v * (th - 1.0)).round() as u32;
            let p = *tex.get_pixel(tx, ty);
            let shaded = image::Rgba([
                (p[0] as f32 * shade) as u8,
                (p[1] as f32 * shade) as u8,
                (p[2] as f32 * shade) as u8,
                p[3],
            ]);
            out.put_pixel(x, y, shaded);
        }
    }
}

/// The named GUI/container/font regions to pack into the UI atlas, each mapped
/// to its jar entry (relative to `assets/minecraft/`).
const UI_TEXTURES: &[(&str, &str)] = &[
    // HUD sprites.
    ("hud_hotbar", "textures/gui/sprites/hud/hotbar.png"),
    (
        "hud_hotbar_selection",
        "textures/gui/sprites/hud/hotbar_selection.png",
    ),
    (
        "hud_heart_container",
        "textures/gui/sprites/hud/heart/container.png",
    ),
    ("hud_heart_full", "textures/gui/sprites/hud/heart/full.png"),
    ("hud_heart_half", "textures/gui/sprites/hud/heart/half.png"),
    ("hud_crosshair", "textures/gui/sprites/hud/crosshair.png"),
    // Container slot + backgrounds.
    ("slot", "textures/gui/sprites/container/slot.png"),
    ("inventory_bg", "textures/gui/container/inventory.png"),
    ("crafting_bg", "textures/gui/container/crafting_table.png"),
    ("furnace_bg", "textures/gui/container/furnace.png"),
    ("chest_bg", "textures/gui/container/generic_54.png"),
    // Font (for item counts).
    ("font", "textures/font/ascii.png"),
];

/// Build the UI atlas: pack every item icon plus the named GUI/container/font
/// regions into one image, returning the atlas, a name→UV map, and an item→UV map.
#[allow(clippy::type_complexity)]
fn build_ui_atlas(
    archive: &mut zip::ZipArchive<File>,
    face_textures: &HashMap<BlockState, FaceTextures>,
    extra: &HashMap<String, image::RgbaImage>,
) -> Result<
    (
        image::RgbaImage,
        HashMap<String, [f32; 4]>,
        HashMap<Item, [f32; 4]>,
    ),
    String,
> {
    let mut entries: Vec<(String, image::RgbaImage)> = Vec::new();
    let mut item_names: HashMap<Item, String> = HashMap::new();

    // Item icons. Block items get an isometric "3D" sprite (Minecraft renders
    // block items as a cube from an elevated 45° angle); non-block items keep
    // their flat sprite. The chest is an exception (flat front texture).
    for (item, def) in item_defs() {
        if def.id == "minecraft:air" {
            continue; // air has no item form
        }
        let (name, img): (String, image::RgbaImage) = if def.id == "minecraft:chest" {
            let name = "entity/chest/normal".to_string();
            let Some(img) = load_texture_image(archive, &name) else {
                continue;
            };
            (name, img)
        } else if let Some(state) = def.block {
            let Some(ft) = face_textures.get(&state) else {
                continue;
            };
            let half_height = matches!(
                sunset_core::block::block_def(state).shape,
                sunset_core::block::Shape::SlabBottom | sunset_core::block::Shape::SlabTop
            );
            let Some(img) = block_item_isometric(archive, ft, half_height) else {
                continue;
            };
            (format!("item3d:{}", def.id), img)
        } else {
            let Some(name) = def.id.strip_prefix("minecraft:").map(|s| format!("item/{s}"))
            else {
                continue;
            };
            let Some(img) = load_texture_image(archive, &name) else {
                continue;
            };
            (name, img)
        };
        // Dedupe: load each unique texture once, but remember every item's name.
        item_names.insert(item, name.clone());
        if !entries.iter().any(|(n, _)| n == &name) {
            entries.push((name.clone(), img));
        }
    }

    // Named GUI/container/font regions.
    for (key, entry) in UI_TEXTURES {
        let Ok(bytes) = read_jar_entry(archive, &format!("assets/minecraft/{entry}")) else {
            continue;
        };
        let Ok(img) = image::load_from_memory(&bytes) else {
            continue;
        };
        let img = img.to_rgba8();
        // Container backgrounds are 256×256 with the GUI in the top-left; crop
        // to the drawable region so the stored rect is exactly what gets drawn.
        let img = match *key {
            "chest_bg" => crop(&img, 176, 222),
            "inventory_bg" | "crafting_bg" | "furnace_bg" => crop(&img, 176, 166),
            _ => img,
        };
        entries.push((key.to_string(), img));
    }

    // Runtime-registered custom textures (mod content), merged into the atlas.
    for (name, img) in extra {
        entries.push((name.clone(), img.clone()));
    }

    // A solid 1×1 white pixel, used to tint solid-color UI elements.
    let mut white = image::RgbaImage::new(1, 1);
    white.put_pixel(0, 0, image::Rgba([255, 255, 255, 255]));
    entries.push(("white".to_string(), white));

    let (atlas, uvs) = pack_atlas(entries);
    let item_icons = item_names
        .into_iter()
        .filter_map(|(item, name)| uvs.get(&name).map(|uv| (item, *uv)))
        .collect();

    Ok((atlas, uvs, item_icons))
}

/// Crop an image to its top-left `w`×`h` region.
fn crop(img: &image::RgbaImage, w: u32, h: u32) -> image::RgbaImage {
    image::imageops::crop_imm(img, 0, 0, w, h).to_image()
}

/// Pack a set of named images into a single atlas (simple shelf packing),
/// returning the atlas and a name → normalized UV-rect map.
fn pack_atlas(
    entries: Vec<(String, image::RgbaImage)>,
) -> (image::RgbaImage, HashMap<String, [f32; 4]>) {
    const MAX_W: u32 = 1024;
    /// 1px transparent gap between regions so linear filtering does not bleed
    /// across region boundaries (the atlas edges are handled by ClampToEdge).
    const PAD: u32 = 1;

    let mut order: Vec<usize> = (0..entries.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(entries[i].1.height()));

    let mut x = 0u32;
    let mut y = 0u32;
    let mut row_h = 0u32;
    let mut w = 1u32;
    let mut h = 1u32;
    let mut placed: Vec<(usize, u32, u32)> = Vec::with_capacity(entries.len());

    for &i in &order {
        let img = &entries[i].1;
        let (iw, ih) = (img.width(), img.height());
        if iw > MAX_W {
            continue;
        }
        if x > 0 && x + iw > MAX_W {
            x = 0;
            y += row_h + PAD;
            row_h = 0;
        }
        placed.push((i, x, y));
        x += iw + PAD;
        row_h = row_h.max(ih);
        w = w.max(x.saturating_sub(PAD));
        h = h.max(y + row_h);
    }

    let mut atlas = image::RgbaImage::new(w, h);
    let mut uvs = HashMap::new();
    for (i, px, py) in placed {
        let (name, img) = &entries[i];
        image::imageops::replace(&mut atlas, img, px as i64, py as i64);
        let u0 = px as f32 / w as f32;
        let v0 = py as f32 / h as f32;
        let u1 = (px + img.width()) as f32 / w as f32;
        let v1 = (py + img.height()) as f32 / h as f32;
        uvs.insert(name.clone(), [u0, v0, u1, v1]);
    }

    (atlas, uvs)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test (skipped when no local Minecraft install exists): the UI atlas
    /// must resolve the HUD sprites, the font, and at least some item icons.
    #[test]
    fn ui_atlas_resolves_sprites_and_items() {
        let Some(dir) = find_minecraft_dir() else {
            return;
        };
        let loaded = load_assets(&dir, &HashMap::new()).expect("load assets");
        assert!(loaded.ui_uvs.contains_key("hud_hotbar"));
        assert!(loaded.ui_uvs.contains_key("hud_hotbar_selection"));
        assert!(loaded.ui_uvs.contains_key("hud_heart_full"));
        assert!(loaded.ui_uvs.contains_key("slot"));
        assert!(loaded.ui_uvs.contains_key("inventory_bg"));
        assert!(loaded.ui_uvs.contains_key("crafting_bg"));
        assert!(loaded.ui_uvs.contains_key("furnace_bg"));
        assert!(loaded.ui_uvs.contains_key("chest_bg"));
        assert!(loaded.ui_uvs.contains_key("font"));
        assert!(loaded.ui_uvs.contains_key("white"));
        // Every item except air should have an icon.
        let expected = sunset_core::item::item_defs()
            .into_iter()
            .filter(|(_, d)| d.id != "minecraft:air")
            .count();
        assert_eq!(loaded.item_icons.len(), expected);
    }
}
