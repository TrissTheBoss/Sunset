//! The 2D user interface: a screen-space quad renderer plus the HUD (hotbar,
//! health, crosshair) and the container screens (inventory, crafting, furnace,
//! chest). Everything is drawn from the UI texture atlas built in `assets`.

use std::collections::HashMap;

use sunset_core::item::{Item, ItemStack};
use sunset_core::player::{Player, HOTBAR_SIZE, INVENTORY_SIZE};

/// A single UI vertex: screen-space position (px, top-left origin), normalized
/// atlas UV, and an RGBA tint.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: u32,
}

const UI_VERTEX_LAYOUT: wgpu::VertexBufferLayout = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<UiVertex>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
            offset: 8,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
            offset: 16,
            shader_location: 2,
            format: wgpu::VertexFormat::Uint32,
        },
    ],
};

const UI_SHADER: &str = r#"
struct Uniforms {
    screen: vec2<f32>,
}
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: u32,
}
struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}
fn unpack_rgba8(c: u32) -> vec4<f32> {
    return vec4<f32>(
        f32((c >> 24u) & 0xffu) / 255.0,
        f32((c >> 16u) & 0xffu) / 255.0,
        f32((c >> 8u) & 0xffu) / 255.0,
        f32(c & 0xffu) / 255.0,
    );
}
@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = vec4<f32>(
        in.pos.x / u.screen.x * 2.0 - 1.0,
        1.0 - in.pos.y / u.screen.y * 2.0,
        0.0,
        1.0,
    );
    out.uv = in.uv;
    out.color = unpack_rgba8(in.color);
    return out;
}
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.uv) * in.color;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UiUniforms {
    screen: [f32; 2],
}

/// Pack `[r, g, b, a]` into the `0xRRGGBBAA` u32 the UI shader expects.
#[inline]
pub fn pack_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

/// Resolved UI texture data: the atlas dimensions plus the named-region and
/// item-icon UV maps (normalized, from `assets::build_ui_atlas`).
pub struct UiAssets {
    pub atlas_size: (u32, u32),
    pub uvs: HashMap<String, [f32; 4]>,
    pub item_icons: HashMap<Item, [f32; 4]>,
}

/// A named UV rect, or an empty rect if missing.
fn uv(a: &UiAssets, name: &str) -> [f32; 4] {
    a.uvs.get(name).copied().unwrap_or([0.0, 0.0, 0.0, 0.0])
}

/// Accumulates quads into vertex/index buffers (rebuilt each frame).
pub struct UiBuilder {
    pub vertices: Vec<UiVertex>,
    pub indices: Vec<u32>,
}

impl UiBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Draw a textured quad at `(x, y)` with size `(w, h)` (screen px), sampling
    /// the normalized atlas rect `uv`, tinted by `color`.
    #[allow(clippy::too_many_arguments)]
    pub fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], color: u32) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&[
            UiVertex { pos: [x, y], uv: [uv[0], uv[1]], color },
            UiVertex { pos: [x + w, y], uv: [uv[2], uv[1]], color },
            UiVertex { pos: [x + w, y + h], uv: [uv[2], uv[3]], color },
            UiVertex { pos: [x, y + h], uv: [uv[0], uv[3]], color },
        ]);
        self.indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
        ]);
    }

    /// Draw a solid-colour quad (using the white pixel, tinted by `color`).
    pub fn solid(&mut self, a: &UiAssets, x: f32, y: f32, w: f32, h: f32, color: u32) {
        self.quad(x, y, w, h, uv(a, "white"), color);
    }
}

impl Default for UiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The GUI scale (Minecraft "auto"): grows with the smallest screen dimension.
pub fn gui_scale(width: u32, height: u32) -> u32 {
    let min = width.min(height);
    if min >= 1200 {
        4
    } else if min >= 800 {
        3
    } else if min >= 400 {
        2
    } else {
        1
    }
}

/// The atlas UV rect of glyph `c` in the ASCII font (8×8 cells in a 16×16 grid).
pub fn glyph_uv(a: &UiAssets, c: u8) -> Option<[f32; 4]> {
    let font = *a.uvs.get("font")?;
    let (aw, ah) = (a.atlas_size.0 as f32, a.atlas_size.1 as f32);
    let col = (c % 16) as f32;
    let row = (c / 16) as f32;
    Some([
        font[0] + col * 8.0 / aw,
        font[1] + row * 8.0 / ah,
        font[0] + (col + 1.0) * 8.0 / aw,
        font[1] + (row + 1.0) * 8.0 / ah,
    ])
}

/// Draw a right-aligned string of digits at `(x, y)`, scaled by `scale` (each
/// glyph is an 8×8 cell). `x` is the right edge; glyphs advance leftward.
fn draw_number(
    b: &mut UiBuilder,
    a: &UiAssets,
    x: f32,
    y: f32,
    scale: f32,
    text: &str,
    color: u32,
) {
    let mut cx = x;
    for ch in text.chars().rev() {
        let Some(g) = glyph_uv(a, ch as u8) else {
            continue;
        };
        let w = 8.0 * scale;
        cx -= w;
        b.quad(cx, y, w, w, g, color);
    }
}

/// The on-screen width (px) of `text` rendered at `scale`.
pub fn text_width(text: &str, scale: f32) -> f32 {
    text.chars().count() as f32 * 8.0 * scale
}

/// Draw a left-to-right string of ASCII glyphs at `(x, y)`, scaled by `scale`.
pub fn draw_text(
    b: &mut UiBuilder,
    a: &UiAssets,
    x: f32,
    y: f32,
    scale: f32,
    text: &str,
    color: u32,
) {
    let mut cx = x;
    for ch in text.chars() {
        let Some(g) = glyph_uv(a, ch as u8) else {
            continue;
        };
        let w = 8.0 * scale;
        b.quad(cx, y, w, w, g, color);
        cx += w;
    }
}

/// Draw an item icon at `(x, y)` with square side length `size` (screen px).
pub fn draw_item_icon(b: &mut UiBuilder, a: &UiAssets, x: f32, y: f32, size: f32, item: Item) {
    if item == sunset_core::item::EMPTY_ITEM {
        return;
    }
    let Some(rect) = a.item_icons.get(&item) else {
        return;
    };
    b.quad(x, y, size, size, *rect, pack_rgba(255, 255, 255, 255));
}

/// Draw a single inventory slot: background, item icon, and count.
pub fn draw_slot(
    b: &mut UiBuilder,
    a: &UiAssets,
    x: f32,
    y: f32,
    size: f32,
    stack: &ItemStack,
) {
    b.quad(x, y, size, size, uv(a, "slot"), pack_rgba(255, 255, 255, 255));
    draw_slot_contents(b, a, x, y, size, stack);
}

/// Draw a slot's item icon + count (no background), used where the background
/// is painted separately (e.g. the hotbar strip).
fn draw_slot_contents(
    b: &mut UiBuilder,
    a: &UiAssets,
    x: f32,
    y: f32,
    size: f32,
    stack: &ItemStack,
) {
    if stack.is_empty() {
        return;
    }
    let pad = size * 0.0625; // 1px at scale 1
    draw_item_icon(b, a, x + pad, y + pad, size - pad * 2.0, stack.item);
    if stack.count > 1 {
        let fs = size / 18.0; // 8px glyph at 18px slot
        draw_number(
            b,
            a,
            x + size - pad,
            y + size - pad,
            fs,
            &stack.count.to_string(),
            pack_rgba(255, 255, 255, 255),
        );
    }
}

/// Draw the hotbar strip centred at the bottom of the screen.
pub fn draw_hotbar(
    b: &mut UiBuilder,
    a: &UiAssets,
    inventory: &[ItemStack; INVENTORY_SIZE],
    selected: usize,
    screen: (u32, u32),
) {
    let s = gui_scale(screen.0, screen.1) as f32;
    let (sw, sh) = (screen.0 as f32, screen.1 as f32);
    let hotbar = uv(a, "hud_hotbar"); // 182x22
    let hw = 182.0 * s;
    let hh = 22.0 * s;
    let x = (sw - hw) / 2.0;
    let y = sh - hh;
    b.quad(x, y, hw, hh, hotbar, pack_rgba(255, 255, 255, 255));

    for (i, stack) in inventory.iter().enumerate().take(HOTBAR_SIZE) {
        let sx = x + (i as f32 * 20.0 + 2.0) * s;
        let sy = y + 2.0 * s;
        let slot = 18.0 * s;
        draw_slot_contents(b, a, sx, sy, slot, stack);
        if i == selected {
            let sel = uv(a, "hud_hotbar_selection"); // 24x23
            b.quad(
                sx - 3.0 * s,
                sy - 2.0 * s,
                24.0 * s,
                23.0 * s,
                sel,
                pack_rgba(255, 255, 255, 255),
            );
        }
    }
}

/// Draw the health hearts (10 hearts, half-heart granularity) above the hotbar.
pub fn draw_health(b: &mut UiBuilder, a: &UiAssets, health: f32, screen: (u32, u32)) {
    let s = gui_scale(screen.0, screen.1) as f32;
    let (sw, sh) = (screen.0 as f32, screen.1 as f32);
    let hotbar_w = 182.0 * s;
    let x = (sw - hotbar_w) / 2.0 + 1.0 * s;
    let y = sh - (22.0 * s) - (10.0 * s);

    let full = (health / 2.0).floor() as i32;
    let half = (health as i32) % 2;
    for i in 0..10 {
        let hx = x + (i as f32 * 8.0) * s;
        let hs = 9.0 * s;
        b.quad(hx, y, hs, hs, uv(a, "hud_heart_container"), pack_rgba(255, 255, 255, 255));
        if i < full {
            b.quad(hx, y, hs, hs, uv(a, "hud_heart_full"), pack_rgba(255, 255, 255, 255));
        } else if i == full && half == 1 {
            b.quad(hx, y, hs, hs, uv(a, "hud_heart_half"), pack_rgba(255, 255, 255, 255));
        }
    }
}

/// Draw the crosshair centred on screen.
pub fn draw_crosshair(b: &mut UiBuilder, a: &UiAssets, screen: (u32, u32)) {
    let s = gui_scale(screen.0, screen.1) as f32;
    let (sw, sh) = (screen.0 as f32, screen.1 as f32);
    let c = uv(a, "hud_crosshair"); // 15x15
    let size = 15.0 * s;
    b.quad(
        sw / 2.0 - size / 2.0,
        sh / 2.0 - size / 2.0,
        size,
        size,
        c,
        pack_rgba(255, 255, 255, 255),
    );
}

/// A full-screen HUD (hotbar + health + crosshair).
pub fn draw_hud(
    b: &mut UiBuilder,
    a: &UiAssets,
    inventory: &[ItemStack; INVENTORY_SIZE],
    selected: usize,
    health: f32,
    screen: (u32, u32),
) {
    draw_crosshair(b, a, screen);
    draw_hotbar(b, a, inventory, selected, screen);
    draw_health(b, a, health, screen);
}

// ---------------------------------------------------------------------------
// Container screens
// ---------------------------------------------------------------------------

/// A container background plus the positions of its container-only slots.
pub struct ContainerLayout {
    pub bg: &'static str,
    /// Background height in GUI px (166 for most, 222 for the chest).
    pub bg_h: f32,
    /// Container-only slot positions `(x, y)` in GUI px (scale 1), in the order
    /// their contents are supplied.
    pub top_slots: &'static [(f32, f32)],
}

/// The inventory screen (no container-only slots; just the player inventory).
pub const INVENTORY_LAYOUT: ContainerLayout = ContainerLayout {
    bg: "inventory_bg",
    bg_h: 166.0,
    top_slots: &[],
};

/// The 3×3 crafting grid + result slot.
pub const CRAFTING_LAYOUT: ContainerLayout = ContainerLayout {
    bg: "crafting_bg",
    bg_h: 166.0,
    top_slots: &[
        (30.0, 17.0),
        (48.0, 17.0),
        (66.0, 17.0),
        (30.0, 35.0),
        (48.0, 35.0),
        (66.0, 35.0),
        (30.0, 53.0),
        (48.0, 53.0),
        (66.0, 53.0),
        (124.0, 35.0), // result
    ],
};

/// The furnace's input, fuel, and output slots.
pub const FURNACE_LAYOUT: ContainerLayout = ContainerLayout {
    bg: "furnace_bg",
    bg_h: 166.0,
    top_slots: &[(56.0, 17.0), (56.0, 53.0), (116.0, 35.0)],
};

/// The single chest's 27 slots.
pub const CHEST_LAYOUT: ContainerLayout = ContainerLayout {
    bg: "chest_bg",
    bg_h: 222.0,
    top_slots: &[
        (8.0, 18.0),
        (26.0, 18.0),
        (44.0, 18.0),
        (62.0, 18.0),
        (80.0, 18.0),
        (98.0, 18.0),
        (116.0, 18.0),
        (134.0, 18.0),
        (152.0, 18.0),
        (8.0, 36.0),
        (26.0, 36.0),
        (44.0, 36.0),
        (62.0, 36.0),
        (80.0, 36.0),
        (98.0, 36.0),
        (116.0, 36.0),
        (134.0, 36.0),
        (152.0, 36.0),
        (8.0, 54.0),
        (26.0, 54.0),
        (44.0, 54.0),
        (62.0, 54.0),
        (80.0, 54.0),
        (98.0, 54.0),
        (116.0, 54.0),
        (134.0, 54.0),
        (152.0, 54.0),
    ],
};

/// Draw a container screen: the background, its container-only slots (whose
/// contents are in `top`), the player main inventory (27) and hotbar (9).
/// Returns the top-left screen-space position of every slot, in draw order:
/// container slots, then main inventory (row-major), then hotbar.
pub fn draw_container_screen(
    b: &mut UiBuilder,
    a: &UiAssets,
    layout: &ContainerLayout,
    top: &[ItemStack],
    player: &Player,
    screen: (u32, u32),
) -> Vec<(f32, f32)> {
    let s = gui_scale(screen.0, screen.1) as f32;
    let (sw, sh) = (screen.0 as f32, screen.1 as f32);
    let (bw, bh) = (176.0 * s, layout.bg_h * s);
    let ox = (sw - bw) / 2.0;
    let oy = (sh - bh) / 2.0;
    b.quad(ox, oy, bw, bh, uv(a, layout.bg), pack_rgba(255, 255, 255, 255));

    let mut slots: Vec<(f32, f32)> = Vec::with_capacity(top.len() + INVENTORY_SIZE);

    // Container-only slots.
    for (i, &(gx, gy)) in layout.top_slots.iter().enumerate() {
        let x = ox + gx * s;
        let y = oy + gy * s;
        draw_slot(b, a, x, y, 18.0 * s, top.get(i).unwrap_or(&ItemStack::EMPTY));
        slots.push((x, y));
    }

    // Player main inventory (18 slots, 2 rows of 9), then the hotbar.
    for row in 0..2 {
        for col in 0..9 {
            let x = ox + (8.0 + col as f32 * 18.0) * s;
            let y = oy + (84.0 + row as f32 * 18.0) * s;
            draw_slot(b, a, x, y, 18.0 * s, &player.inventory[9 + col + row * 9]);
            slots.push((x, y));
        }
    }

    // Hotbar (9 slots).
    for col in 0..9 {
        let x = ox + (8.0 + col as f32 * 18.0) * s;
        let y = oy + 142.0 * s;
        draw_slot(b, a, x, y, 18.0 * s, &player.inventory[col]);
        slots.push((x, y));
    }

    slots
}

/// The active UI screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    None,
    Inventory,
    Crafting,
    Furnace,
    Chest,
}

/// A mouse click on a slot, used to move items between the cursor and the slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Click {
    Left,
    Right,
}

/// Apply a click to a slot, using Minecraft's pick-up / place / swap semantics.
pub fn click_slot(cursor: &mut ItemStack, slot: &mut ItemStack, click: Click) {
    match click {
        Click::Left => {
            if cursor.is_empty() {
                *cursor = *slot;
                *slot = ItemStack::EMPTY;
            } else if slot.item == cursor.item {
                let room = cursor.max_stack() - cursor.count;
                let take = room.min(slot.count);
                cursor.count += take;
                slot.count -= take;
                if slot.count == 0 {
                    *slot = ItemStack::EMPTY;
                }
            } else {
                std::mem::swap(cursor, slot);
            }
        }
        Click::Right => {
            if cursor.is_empty() {
                if !slot.is_empty() {
                    let take = slot.count.div_ceil(2);
                    cursor.item = slot.item;
                    cursor.count = take;
                    slot.count -= take;
                    if slot.count == 0 {
                        *slot = ItemStack::EMPTY;
                    }
                }
            } else if slot.is_empty() {
                slot.item = cursor.item;
                slot.count = 1;
                cursor.count -= 1;
                if cursor.count == 0 {
                    *cursor = ItemStack::EMPTY;
                }
            } else if slot.item == cursor.item {
                if slot.count < slot.max_stack() {
                    slot.count += 1;
                    cursor.count -= 1;
                    if cursor.count == 0 {
                        *cursor = ItemStack::EMPTY;
                    }
                }
            } else {
                std::mem::swap(cursor, slot);
            }
        }
    }
}

/// The GPU-side UI renderer (pipeline, bind group, growable buffers).
pub struct UiRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_cap: usize,
    index_cap: usize,
}

impl UiRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &image::RgbaImage,
        format: wgpu::TextureFormat,
    ) -> Self {
        let (aw, ah) = atlas.dimensions();
        let size = wgpu::Extent3d {
            width: aw,
            height: ah,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ui atlas"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(atlas.as_raw()),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * aw),
                rows_per_image: Some(ah),
            },
            size,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ui sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui uniforms"),
            size: std::mem::size_of::<UiUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ui bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui shader"),
            source: wgpu::ShaderSource::Wgsl(UI_SHADER.into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(UI_VERTEX_LAYOUT)],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui vertex"),
            size: 4,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui index"),
            size: 4,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            vertex_cap: 0,
            index_cap: 0,
        }
    }

    /// Draw the accumulated UI quads into the active render pass.
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
        screen: (u32, u32),
        builder: &UiBuilder,
    ) {
        if builder.indices.is_empty() {
            return;
        }
        self.ensure_capacity(device, &builder.vertices, &builder.indices);
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[UiUniforms {
                screen: [screen.0 as f32, screen.1 as f32],
            }]),
        );
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&builder.vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&builder.indices));

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..builder.indices.len() as u32, 0, 0..1);
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, verts: &[UiVertex], idx: &[u32]) {
        let vbytes = std::mem::size_of_val(verts) as u64;
        if self.vertex_cap < vbytes as usize {
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ui vertex"),
                size: vbytes.max(4),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_cap = vbytes as usize;
        }
        let ibytes = std::mem::size_of_val(idx) as u64;
        if self.index_cap < ibytes as usize {
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ui index"),
                size: ibytes.max(4),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.index_cap = ibytes as usize;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sunset_core::item::item_by_id;

    fn stack(item: Item, count: u32) -> ItemStack {
        ItemStack::new(item, count)
    }

    #[test]
    fn gui_scale_steps() {
        assert_eq!(gui_scale(1280, 720), 2);
        assert_eq!(gui_scale(1920, 1080), 3);
        assert_eq!(gui_scale(2560, 1440), 4);
        assert_eq!(gui_scale(320, 240), 1);
    }

    #[test]
    fn left_click_picks_up_all() {
        let stone = item_by_id("minecraft:stone").unwrap();
        let mut cursor = ItemStack::EMPTY;
        let mut slot = stack(stone, 5);
        click_slot(&mut cursor, &mut slot, Click::Left);
        assert_eq!(cursor.count, 5);
        assert!(slot.is_empty());
    }

    #[test]
    fn left_click_stacks_same_item() {
        let stone = item_by_id("minecraft:stone").unwrap();
        let mut cursor = stack(stone, 60);
        let mut slot = stack(stone, 10);
        click_slot(&mut cursor, &mut slot, Click::Left);
        assert_eq!(cursor.count, 64); // capped at max stack
        assert_eq!(slot.count, 6);
    }

    #[test]
    fn left_click_swaps_different_items() {
        let stone = item_by_id("minecraft:stone").unwrap();
        let dirt = item_by_id("minecraft:dirt").unwrap();
        let mut cursor = stack(stone, 3);
        let mut slot = stack(dirt, 7);
        click_slot(&mut cursor, &mut slot, Click::Left);
        assert_eq!(cursor.item, dirt);
        assert_eq!(cursor.count, 7);
        assert_eq!(slot.item, stone);
        assert_eq!(slot.count, 3);
    }

    #[test]
    fn right_click_picks_up_half() {
        let stone = item_by_id("minecraft:stone").unwrap();
        let mut cursor = ItemStack::EMPTY;
        let mut slot = stack(stone, 5);
        click_slot(&mut cursor, &mut slot, Click::Right);
        assert_eq!(cursor.count, 3); // ceil(5/2)
        assert_eq!(slot.count, 2);
    }

    #[test]
    fn right_click_places_one() {
        let stone = item_by_id("minecraft:stone").unwrap();
        let mut cursor = stack(stone, 4);
        let mut slot = ItemStack::EMPTY;
        click_slot(&mut cursor, &mut slot, Click::Right);
        assert_eq!(slot.count, 1);
        assert_eq!(cursor.count, 3);
    }
}
