//! Sunset — a raw `winit` + `wgpu` renderer over the authoritative
//! `sunset-core` simulation. This is the only renderer; the former Bevy
//! adapter was removed in favor of this performance path. See
//! RAW_RENDERER_PLAN.md.

mod assets;
mod mesh;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Quat, Vec2, Vec3};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

use sunset_core::block::{block_def, BlockState};
use sunset_core::chunk::{CHUNK_HEIGHT, CHUNK_SIZE};
use sunset_core::fixture;
use sunset_core::intent::{BlockAction, PlayerIntent};
use sunset_core::physics;
use sunset_core::player::{Player, PLAYER_EYE_HEIGHT, PLAYER_HEIGHT};
use sunset_core::save;
use sunset_core::simulation::Simulation;
use sunset_core::world::World;

use assets::FaceTextures;
use mesh::{MeshData, Vertex};

/// Look sensitivity in radians per pixel of mouse motion.
const SENSITIVITY: f32 = 0.0025;
/// Fixed simulation tick (20 Hz), matching the former Bevy adapter.
const FIXED_DT: f32 = 1.0 / 20.0;
/// Block interaction reach distance (blocks).
const REACH: f32 = 5.0;
/// Number of recent frame durations kept for average / 0.1%-low FPS.
const FRAME_WINDOW: usize = 1000;
/// Warm-up seconds before benchmark frame times are recorded.
const WARMUP_SECS: f32 = 2.0;
/// Where the development save lives (a versioned `.sunset` file).
const SAVE_PATH: &str = "saves/building-yard.sunset";

/// Per-frame camera data: the world view-projection plus the outline's model
/// matrix (the outline is a static unit cube scaled/translated per frame).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    outline_model: [[f32; 4]; 4],
}

/// Rolling window of frame durations used to report average and 0.1%-low FPS
/// (mirrors the Bevy adapter's `hud::FrameStats`).
struct FrameStats {
    times: VecDeque<f32>,
}

impl FrameStats {
    fn new() -> Self {
        Self {
            times: VecDeque::with_capacity(FRAME_WINDOW),
        }
    }

    fn record(&mut self, dt: f32) {
        if dt > 0.0 && dt.is_finite() {
            self.times.push_back(dt);
            if self.times.len() > FRAME_WINDOW {
                self.times.pop_front();
            }
        }
    }

    /// Average FPS over the window (frames divided by total time).
    fn average_fps(&self) -> Option<f32> {
        let n = self.times.len();
        if n == 0 {
            return None;
        }
        let total: f32 = self.times.iter().sum();
        if total <= 0.0 {
            return None;
        }
        Some(n as f32 / total)
    }

    /// 0.1%-low FPS: `1 / (99.9th-percentile frame time)`.
    fn low_01_fps(&self) -> Option<f32> {
        let n = self.times.len();
        if n == 0 {
            return None;
        }
        let mut sorted: Vec<f32> = self.times.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((n as f32 * 0.999).ceil() as usize)
            .saturating_sub(1)
            .min(n - 1);
        let dt = sorted[idx];
        if dt <= 0.0 {
            return None;
        }
        Some(1.0 / dt)
    }
}

/// Average + 0.1%-low FPS over an arbitrary frame-time slice (benchmark window).
fn summarize(times: &[f32]) -> Option<(f32, f32)> {
    let n = times.len();
    if n == 0 {
        return None;
    }
    let total: f32 = times.iter().sum();
    if total <= 0.0 {
        return None;
    }
    let mut sorted = times.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((n as f32 * 0.999).ceil() as usize)
        .saturating_sub(1)
        .min(n - 1);
    Some((n as f32 / total, 1.0 / sorted[idx]))
}

/// Map a number key (1..=9) to a 0-based hotbar slot.
fn digit_slot(code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::Digit1 => Some(0),
        KeyCode::Digit2 => Some(1),
        KeyCode::Digit3 => Some(2),
        KeyCode::Digit4 => Some(3),
        KeyCode::Digit5 => Some(4),
        KeyCode::Digit6 => Some(5),
        KeyCode::Digit7 => Some(6),
        KeyCode::Digit8 => Some(7),
        KeyCode::Digit9 => Some(8),
        _ => None,
    }
}

/// Load the save, or generate the authored fixture on first launch.
fn load_or_generate() -> (World, Player) {
    if let Ok(json) = std::fs::read_to_string(SAVE_PATH) {
        if let Ok((world, player)) = save::from_json(&json) {
            return (world, player);
        }
    }
    let world = fixture::building_yard();
    let player = Player::new(Vec3::new(8.0, 5.5, 16.0), 0.5, -0.25);
    (world, player)
}

/// The interleaved vertex layout: position(f32x3) + uv(f32x2) + color(f32x4).
const VERTEX_LAYOUT: wgpu::VertexBufferLayout = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
            offset: 12,
            shader_location: 1,
            format: wgpu::VertexFormat::Uint16x2,
        },
        wgpu::VertexAttribute {
            offset: 16,
            shader_location: 2,
            format: wgpu::VertexFormat::Uint32,
        },
        wgpu::VertexAttribute {
            offset: 20,
            shader_location: 3,
            format: wgpu::VertexFormat::Uint16x4,
        },
    ],
};

const SHADER: &str = r#"
struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<u32>,
    @location(2) color: u32,
    @location(3) rect: vec4<u32>,
};
struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) rect: vec4<f32>,
};
struct Camera {
    view_proj: mat4x4<f32>,
    outline_model: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: Camera;
@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.uv = vec2<f32>(in.uv) / 256.0;
    out.color = unpack_rgba8(in.color);
    out.rect = vec4<f32>(in.rect) / 65535.0;
    return out;
}
fn unpack_rgba8(c: u32) -> vec4<f32> {
    let r = f32((c >> 24u) & 0xffu) / 255.0;
    let g = f32((c >> 16u) & 0xffu) / 255.0;
    let b = f32((c >> 8u) & 0xffu) / 255.0;
    let a = f32(c & 0xffu) / 255.0;
    return vec4<f32>(r, g, b, a);
}
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Wrap the tiled UV within the texture's atlas rect so merged rectangles
    // repeat the texture instead of stretching it.
    let u = mix(in.rect.x, in.rect.z, fract(in.uv.x));
    let v = mix(in.rect.y, in.rect.w, fract(in.uv.y));
    return textureSample(tex, samp, vec2<f32>(u, v)) * in.color;
}
"#;

/// A single line-list vertex (highlight outline): position only.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LineVertex {
    position: [f32; 3],
}

/// Vertex layout for the highlight outline (line list).
const LINE_LAYOUT: wgpu::VertexBufferLayout = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[wgpu::VertexAttribute {
        offset: 0,
        shader_location: 0,
        format: wgpu::VertexFormat::Float32x3,
    }],
};

/// Unlit line shader: transform the unit-cube outline by its model matrix and
/// the view-projection, then emit a solid black pixel.
const LINE_SHADER: &str = r#"
struct Camera {
    view_proj: mat4x4<f32>,
    outline_model: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: Camera;
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return camera.view_proj * camera.outline_model * vec4<f32>(position, 1.0);
}
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
"#;

/// GPU-side mesh: the vertex/index buffers for one drawable (opaque or
/// transparent). The pipeline is shared and stored separately.
struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

/// GPU-side per-chunk meshes (opaque + transparent).
struct GpuChunk {
    opaque: Mesh,
    transparent: Mesh,
}

/// CPU-side per-chunk meshes produced by the mesher.
struct ChunkMesh {
    opaque: MeshData,
    transparent: MeshData,
}

/// Everything needed to draw the world: shared pipelines plus per-chunk meshes.
struct Renderer {
    opaque_pipeline: wgpu::RenderPipeline,
    transparent_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    depth_view: wgpu::TextureView,
    line_pipeline: wgpu::RenderPipeline,
    line_vertex_buffer: wgpu::Buffer,
    chunks: HashMap<(i32, i32), GpuChunk>,
}

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        atlas: &image::RgbaImage,
        chunks: &HashMap<(i32, i32), ChunkMesh>,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        // Upload the atlas (or a 1x1 white fallback for flat colors).
        let (tex_width, tex_height) = if atlas.width() > 0 {
            (atlas.width(), atlas.height())
        } else {
            (1, 1)
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas"),
            size: wgpu::Extent3d {
                width: tex_width,
                height: tex_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        if atlas.width() > 0 {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                atlas.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * tex_width),
                    rows_per_image: Some(tex_height),
                },
                wgpu::Extent3d {
                    width: tex_width,
                    height: tex_height,
                    depth_or_array_layers: 1,
                },
            );
        } else {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &[255, 255, 255, 255],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
        }

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let depth_view = create_depth_view(device, width, height);

        let opaque_pipeline = create_pipeline(device, &shader, &pipeline_layout, format, None, true);
        let transparent_pipeline = create_pipeline(
            device,
            &shader,
            &pipeline_layout,
            format,
            Some(wgpu::BlendState::ALPHA_BLENDING),
            false,
        );

        let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("line shader"),
            source: wgpu::ShaderSource::Wgsl(LINE_SHADER.into()),
        });
        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("line pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &line_shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(LINE_LAYOUT)],
            },
            fragment: Some(wgpu::FragmentState {
                module: &line_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let line_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("outline"),
            size: (std::mem::size_of::<LineVertex>() * 24) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Upload the static unit-cube outline once; the per-frame transform is
        // applied via the model matrix in the uniform.
        queue.write_buffer(
            &line_vertex_buffer,
            0,
            bytemuck::cast_slice(&unit_cube_outline()),
        );

        // Upload per-chunk meshes.
        let mut gpu_chunks = HashMap::new();
        for (&key, chunk_mesh) in chunks {
            let opaque = create_mesh(device, queue, &chunk_mesh.opaque);
            let transparent = create_mesh(device, queue, &chunk_mesh.transparent);
            gpu_chunks.insert(key, GpuChunk { opaque, transparent });
        }

        Self {
            opaque_pipeline,
            transparent_pipeline,
            bind_group,
            uniform_buffer,
            depth_view,
            line_pipeline,
            line_vertex_buffer,
            chunks: gpu_chunks,
        }
    }

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.depth_view = create_depth_view(device, width, height);
    }

    fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        outline: bool,
        frustum: &Frustum,
        eye: Vec3,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // Opaque pass: clears color + depth and writes depth.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("opaque"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.53,
                            g: 0.81,
                            b: 0.92,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.opaque_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            for (&(cx, cz), chunk) in &self.chunks {
                if !frustum.intersects_chunk(cx, cz) {
                    continue;
                }
                if chunk.opaque.index_count > 0 {
                    pass.set_vertex_buffer(0, chunk.opaque.vertex_buffer.slice(..));
                    pass.set_index_buffer(
                        chunk.opaque.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    pass.draw_indexed(0..chunk.opaque.index_count, 0, 0..1);
                }
            }
        }

        // Transparent pass: loads the opaque color/depth, alpha-blends, and
        // tests depth without writing it (so glass is correctly occluded and
        // occludes, but doesn't punch holes in the depth buffer).
        if self.chunks.values().any(|c| c.transparent.index_count > 0) {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("transparent"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.transparent_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            // Back-to-front (furthest first) for correct alpha blending.
            let mut transparent: Vec<(i32, i32)> = self
                .chunks
                .keys()
                .copied()
                .filter(|&(cx, cz)| {
                    frustum.intersects_chunk(cx, cz)
                        && self.chunks[&(cx, cz)].transparent.index_count > 0
                })
                .collect();
            transparent.sort_by(|&(ax, az), &(bx, bz)| {
                let da = chunk_dist_sq(eye, ax, az);
                let db = chunk_dist_sq(eye, bx, bz);
                db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
            });
            for (cx, cz) in transparent {
                let chunk = &self.chunks[&(cx, cz)];
                pass.set_vertex_buffer(0, chunk.transparent.vertex_buffer.slice(..));
                pass.set_index_buffer(
                    chunk.transparent.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                pass.draw_indexed(0..chunk.transparent.index_count, 0, 0..1);
            }
        }

        // Highlight pass: draw the targeted block's outline over the scene.
        if outline {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("outline"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.line_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.line_vertex_buffer.slice(..));
            pass.draw(0..24, 0..1);
        }

        queue.submit([encoder.finish()]);
    }

    /// Re-upload a single chunk's meshes after a block edit.
    fn update_chunk(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cx: i32,
        cz: i32,
        mesh: &ChunkMesh,
    ) {
        // Clone the existing buffer handles first so no borrow of `self.chunks`
        // is held across the `insert` below.
        let (ovb, oib, tvb, tib) = match self.chunks.get(&(cx, cz)) {
            Some(c) => (
                Some(c.opaque.vertex_buffer.clone()),
                Some(c.opaque.index_buffer.clone()),
                Some(c.transparent.vertex_buffer.clone()),
                Some(c.transparent.index_buffer.clone()),
            ),
            None => (None, None, None, None),
        };
        let opaque = ensure_mesh(device, queue, ovb.as_ref(), oib.as_ref(), &mesh.opaque);
        let transparent = ensure_mesh(device, queue, tvb.as_ref(), tib.as_ref(), &mesh.transparent);
        self.chunks
            .insert((cx, cz), GpuChunk { opaque, transparent });
    }
}

/// Build a mesh's vertex/index buffers and upload them to the GPU.
fn create_mesh_buffers(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mesh: &MeshData,
) -> (wgpu::Buffer, wgpu::Buffer, u32) {
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("vertex"),
        size: (std::mem::size_of::<Vertex>() * mesh.vertices.len()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&mesh.vertices));
    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("index"),
        size: (std::mem::size_of::<u32>() * mesh.indices.len()) as u64,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&mesh.indices));
    (vertex_buffer, index_buffer, mesh.indices.len() as u32)
}

/// Create a GPU mesh (vertex/index buffers) from CPU mesh data.
fn create_mesh(device: &wgpu::Device, queue: &wgpu::Queue, mesh: &MeshData) -> Mesh {
    let (vertex_buffer, index_buffer, index_count) = create_mesh_buffers(device, queue, mesh);
    Mesh {
        vertex_buffer,
        index_buffer,
        index_count,
    }
}

/// Reuse an existing mesh's buffers when they have enough capacity, else create
/// new ones. Writing new data in place avoids per-edit buffer reallocation.
fn ensure_mesh(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    existing_vb: Option<&wgpu::Buffer>,
    existing_ib: Option<&wgpu::Buffer>,
    mesh: &MeshData,
) -> Mesh {
    let vbytes = (std::mem::size_of::<Vertex>() * mesh.vertices.len()) as u64;
    let ibytes = (std::mem::size_of::<u32>() * mesh.indices.len()) as u64;
    if let (Some(vb), Some(ib)) = (existing_vb, existing_ib) {
        if vb.size() >= vbytes && ib.size() >= ibytes {
            if !mesh.vertices.is_empty() {
                queue.write_buffer(vb, 0, bytemuck::cast_slice(&mesh.vertices));
            }
            if !mesh.indices.is_empty() {
                queue.write_buffer(ib, 0, bytemuck::cast_slice(&mesh.indices));
            }
            return Mesh {
                vertex_buffer: vb.clone(),
                index_buffer: ib.clone(),
                index_count: mesh.indices.len() as u32,
            };
        }
    }
    create_mesh(device, queue, mesh)
}

/// Build a triangle-list render pipeline.
///
/// `blend` selects alpha blending (transparent) vs. `None` (opaque), and
/// `depth_write` controls whether the pass writes to the depth buffer.
#[allow(clippy::too_many_arguments)]
fn create_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    pipeline_layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
    depth_write: bool,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("pipeline"),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[Some(VERTEX_LAYOUT)],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            // No backface culling: the merged quads use outward normals but
            // the index winding is not consistently front-facing. Culling
            // is a later optimization (directional face buckets, M8).
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(depth_write),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn create_depth_view(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// The 24 static line-list vertices of a unit cube (`0..1`), transformed by the
/// per-frame outline model matrix. Written once; never updated per frame.
fn unit_cube_outline() -> [LineVertex; 24] {
    let c: [[f32; 3]; 8] = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ];
    const EDGES: [(usize, usize); 12] = [
        (0, 1),
        (0, 2),
        (1, 3),
        (2, 3),
        (4, 5),
        (4, 6),
        (5, 7),
        (6, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    let mut out = [LineVertex { position: [0.0; 3] }; 24];
    for (i, (a, b)) in EDGES.iter().enumerate() {
        out[i * 2] = LineVertex { position: c[*a] };
        out[i * 2 + 1] = LineVertex { position: c[*b] };
    }
    out
}

/// The model matrix that maps the unit-cube outline onto the collision box of
/// the block at `pos`, slightly expanded so the lines don't z-fight the faces.
fn outline_model_matrix(world: &World, pos: [i32; 3]) -> Option<[[f32; 4]; 4]> {
    let state = world.get_block(pos[0], pos[1], pos[2]);
    let (bmin, bmax) = block_def(state).collision_box()?;
    let base = Vec3::new(pos[0] as f32, pos[1] as f32, pos[2] as f32);
    let e = 0.002;
    let min = base + bmin - Vec3::splat(e);
    let max = base + bmax + Vec3::splat(e);
    Some(
        Mat4::from_scale_rotation_translation(max - min, Quat::IDENTITY, min)
            .to_cols_array_2d(),
    )
}

/// The six clipping planes of a view frustum, extracted from a view-projection
/// matrix (Gribb-Hartmann). Used to cull chunks entirely outside the view.
struct Frustum {
    /// `(a, b, c, d)` planes; a point `p` is inside when `a·p.x+b·p.y+c·p.z+d ≥ 0`.
    planes: [[f32; 4]; 6],
}

impl Frustum {
    fn from_matrix(m: &Mat4) -> Self {
        let c = m.to_cols_array_2d();
        // Rows of the matrix (glam is column-major, so transpose).
        let row = |i: usize| [c[0][i], c[1][i], c[2][i], c[3][i]];
        let r0 = row(0);
        let r1 = row(1);
        let r2 = row(2);
        let r3 = row(3);
        let add = |a: [f32; 4], b: [f32; 4]| [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]];
        let sub = |a: [f32; 4], b: [f32; 4]| [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]];
        // DirectX projection: near is z_clip ≥ 0 (row 2), far is z_clip ≤ w.
        let mut planes = [add(r3, r0), sub(r3, r0), add(r3, r1), sub(r3, r1), r2, sub(r3, r2)];
        for p in &mut planes {
            let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            if len > 0.0 {
                p[0] /= len;
                p[1] /= len;
                p[2] /= len;
                p[3] /= len;
            }
        }
        Self { planes }
    }

    /// Whether a chunk's bounding box is at least partially inside the frustum.
    fn intersects_chunk(&self, cx: i32, cz: i32) -> bool {
        let min = [cx as f32 * CHUNK_SIZE as f32, 0.0, cz as f32 * CHUNK_SIZE as f32];
        let max = [
            (cx + 1) as f32 * CHUNK_SIZE as f32,
            CHUNK_HEIGHT as f32,
            (cz + 1) as f32 * CHUNK_SIZE as f32,
        ];
        self.intersects_aabb(min, max)
    }

    /// Conservative AABB test: `false` only when the box is entirely outside a
    /// plane (the positive corner lies on the negative side).
    fn intersects_aabb(&self, min: [f32; 3], max: [f32; 3]) -> bool {
        for p in &self.planes {
            let px = if p[0] >= 0.0 { max[0] } else { min[0] };
            let py = if p[1] >= 0.0 { max[1] } else { min[1] };
            let pz = if p[2] >= 0.0 { max[2] } else { min[2] };
            if p[0] * px + p[1] * py + p[2] * pz + p[3] < 0.0 {
                return false;
            }
        }
        true
    }
}

/// Squared distance from the eye to a chunk's center.
fn chunk_dist_sq(eye: Vec3, cx: i32, cz: i32) -> f32 {
    let cx_f = (cx as f32 + 0.5) * CHUNK_SIZE as f32;
    let cy_f = CHUNK_HEIGHT as f32 * 0.5;
    let cz_f = (cz as f32 + 0.5) * CHUNK_SIZE as f32;
    let dx = cx_f - eye.x;
    let dy = cy_f - eye.y;
    let dz = cz_f - eye.z;
    dx * dx + dy * dy + dz * dz
}

/// The renderer state: surface/device, the authoritative simulation, and input.
struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Option<Renderer>,
    sim: Simulation,
    intent: PlayerIntent,
    cursor_captured: bool,
    keys: HashSet<KeyCode>,
    mouse_buttons: HashSet<MouseButton>,
    /// Current raycast target (block coords + face normal), for highlight/edit.
    target: Option<([i32; 3], [i32; 3])>,
    /// Texture data retained so the mesh can be rebuilt after an edit.
    face_textures: HashMap<BlockState, FaceTextures>,
    uv: HashMap<String, [f32; 4]>,
    /// Per-chunk revisions at last mesh, used to detect which chunks changed.
    chunk_revisions: HashMap<(i32, i32), u64>,
    last_time: Instant,
    accumulator: f32,
    frames: u64,
    last_log: Instant,
    stats: FrameStats,
    /// App start time, used to time the benchmark warm-up window.
    start: Instant,
    /// Benchmark duration in seconds (from `SUNSET_BENCH_SECS`), if set.
    bench_secs: Option<f32>,
    /// Set once warm-up completes; frame times after this point are recorded.
    bench_start: Option<Instant>,
    bench_times: Vec<f32>,
    /// Whether the window is currently fullscreen (toggled with F11).
    fullscreen: bool,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::default();
        let surface: wgpu::Surface<'static> = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("no compatible GPU adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("could not create device");
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Load the world + atlas + mesh.
        let (face_textures, uv, atlas) =
            match assets::find_minecraft_dir().and_then(|d| assets::load_assets(&d).ok()) {
                Some(loaded) => (loaded.face_textures, loaded.uv, loaded.atlas),
                None => {
                    println!("No Minecraft assets found; using flat colors");
                    (HashMap::new(), HashMap::new(), image::RgbaImage::new(0, 0))
                }
            };
        let (world, player) = load_or_generate();

        // Build per-chunk meshes (never a single combined world mesh, so an
        // edit can re-mesh just the affected chunk and its neighbours).
        let mut chunk_meshes: HashMap<(i32, i32), ChunkMesh> = HashMap::new();
        let mut chunk_revisions: HashMap<(i32, i32), u64> = HashMap::new();
        for chunk in world.chunks() {
            let (opaque, transparent) =
                mesh::build_chunk_meshes(&world, chunk, &face_textures, &uv);
            chunk_meshes.insert((chunk.cx, chunk.cz), ChunkMesh { opaque, transparent });
            chunk_revisions.insert((chunk.cx, chunk.cz), chunk.revision);
        }
        let opaque_verts: usize = chunk_meshes.values().map(|c| c.opaque.vertices.len()).sum();
        let opaque_tris: usize = chunk_meshes.values().map(|c| c.opaque.indices.len() / 3).sum();
        let transparent_tris: usize = chunk_meshes
            .values()
            .map(|c| c.transparent.indices.len() / 3)
            .sum();
        println!(
            "chunks: {} | opaque: {opaque_verts} vertices, {opaque_tris} triangles | transparent: {transparent_tris} triangles",
            chunk_meshes.len()
        );

        let renderer = Renderer::new(
            &device,
            &queue,
            config.width,
            config.height,
            config.format,
            &atlas,
            &chunk_meshes,
        );

        // Authoritative simulation.
        let sim = Simulation::new(world, player);
        let bench_secs = std::env::var("SUNSET_BENCH_SECS")
            .ok()
            .and_then(|s| s.parse::<f32>().ok());

        State {
            window,
            surface,
            device,
            queue,
            config,
            renderer: Some(renderer),
            sim,
            intent: PlayerIntent::default(),
            cursor_captured: false,
            keys: HashSet::new(),
            mouse_buttons: HashSet::new(),
            target: None,
            face_textures,
            uv,
            chunk_revisions,
            last_time: Instant::now(),
            accumulator: 0.0,
            frames: 0,
            last_log: Instant::now(),
            stats: FrameStats::new(),
            start: Instant::now(),
            bench_secs,
            bench_start: None,
            bench_times: Vec::new(),
            fullscreen: false,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        if let Some(r) = &mut self.renderer {
            r.resize(&self.device, size.width, size.height);
        }
    }

    /// Re-mesh any chunks whose revision changed, plus their boundary neighbours
    /// (whose face visibility depends on the edit).
    fn sync_meshes(&mut self) {
        let mut changed: Vec<(i32, i32)> = Vec::new();
        for chunk in self.sim.world.chunks() {
            let key = (chunk.cx, chunk.cz);
            let unchanged = self
                .chunk_revisions
                .get(&key)
                .is_some_and(|r| *r == chunk.revision);
            if !unchanged {
                changed.push(key);
            }
        }
        if changed.is_empty() {
            return;
        }
        let mut dirty: Vec<(i32, i32)> = Vec::new();
        for (cx, cz) in changed {
            dirty.push((cx, cz));
            dirty.push((cx + 1, cz));
            dirty.push((cx - 1, cz));
            dirty.push((cx, cz + 1));
            dirty.push((cx, cz - 1));
        }
        dirty.sort_unstable();
        dirty.dedup();
        for (cx, cz) in dirty {
            self.remesh_chunk(cx, cz);
        }
    }

    /// Re-mesh and re-upload a single chunk.
    fn remesh_chunk(&mut self, cx: i32, cz: i32) {
        let Some(chunk) = self.sim.world.get_chunk(cx, cz) else {
            return;
        };
        let (opaque, transparent) =
            mesh::build_chunk_meshes(&self.sim.world, chunk, &self.face_textures, &self.uv);
        let mesh = ChunkMesh { opaque, transparent };
        if let Some(r) = self.renderer.as_mut() {
            r.update_chunk(&self.device, &self.queue, cx, cz, &mesh);
        }
        self.chunk_revisions.insert((cx, cz), chunk.revision);
    }

    fn render(&mut self) -> bool {
        // Step the authoritative simulation at a fixed 20 Hz.
        let now = Instant::now();
        let dt = (now - self.last_time).as_secs_f32();
        self.last_time = now;
        self.stats.record(dt);
        if self.bench_secs.is_some() {
            match self.bench_start {
                None => {
                    if now.duration_since(self.start).as_secs_f32() >= WARMUP_SECS {
                        self.bench_start = Some(now);
                    }
                }
                Some(_) => self.bench_times.push(dt),
            }
        }
        self.accumulator += dt.min(0.1);
        let mut move_axis = Vec2::ZERO;
        if self.keys.contains(&KeyCode::KeyW) {
            move_axis.y += 1.0;
        }
        if self.keys.contains(&KeyCode::KeyS) {
            move_axis.y -= 1.0;
        }
        if self.keys.contains(&KeyCode::KeyD) {
            move_axis.x += 1.0;
        }
        if self.keys.contains(&KeyCode::KeyA) {
            move_axis.x -= 1.0;
        }
        self.intent.move_axis = move_axis.normalize_or_zero();
        self.intent.jump = self.keys.contains(&KeyCode::Space);

        // Look direction (rotation is authoritative, not interpolated).
        let rot = Quat::from_euler(
            glam::EulerRot::ZYX,
            0.0,
            self.sim.player.yaw,
            self.sim.player.pitch,
        );
        let forward = rot * Vec3::NEG_Z;

        // Raycast from the authoritative eye (matches the Bevy adapter), then
        // map held mouse buttons to a break/place action.
        let eye = self.sim.player.eye_pos();
        self.target = physics::raycast(&self.sim.world, eye, forward, REACH);
        self.intent.block_action = if self.cursor_captured {
            match self.target {
                Some((pos, _face)) if self.mouse_buttons.contains(&MouseButton::Left) => {
                    Some(BlockAction::Break(pos))
                }
                Some((pos, face)) if self.mouse_buttons.contains(&MouseButton::Right) => {
                    Some(BlockAction::Place([
                        pos[0] + face[0],
                        pos[1] + face[1],
                        pos[2] + face[2],
                    ]))
                }
                _ => None,
            }
        } else {
            None
        };

        while self.accumulator >= FIXED_DT {
            self.sim.step(&self.intent, FIXED_DT);
            self.accumulator -= FIXED_DT;
        }
        self.intent.block_action = None;
        self.intent.select_slot = None;

        // Re-mesh only the chunks (and neighbours) that changed.
        self.sync_meshes();

        // Interpolate the camera between the previous and current fixed-tick
        // positions for smooth motion.
        let alpha = (self.accumulator / FIXED_DT).min(1.0);
        let pos = self.sim.prev_pos.lerp(self.sim.player.pos, alpha);
        let eye = pos + Vec3::new(0.0, PLAYER_EYE_HEIGHT - PLAYER_HEIGHT * 0.5, 0.0);
        let up = rot * Vec3::Y;
        let view = glam::camera::rh::view::look_to_mat4(eye, forward, up);
        let aspect = self.config.width as f32 / self.config.height.max(1) as f32;
        let proj = glam::camera::rh::proj::directx::perspective(
            70.0_f32.to_radians(),
            aspect,
            0.1,
            1000.0,
        );
        let vp = proj * view;
        let uniform = CameraUniform {
            view_proj: vp.to_cols_array_2d(),
            outline_model: match self.target {
                Some((pos, _face)) => outline_model_matrix(&self.sim.world, pos)
                    .unwrap_or(Mat4::IDENTITY.to_cols_array_2d()),
                None => Mat4::IDENTITY.to_cols_array_2d(),
            },
        };
        let frustum = Frustum::from_matrix(&vp);

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            _ => {
                self.surface.configure(&self.device, &self.config);
                return false;
            }
        };
        let view_tex = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        if let Some(r) = &self.renderer {
            self.queue.write_buffer(
                &r.uniform_buffer,
                0,
                bytemuck::cast_slice(std::slice::from_ref(&uniform)),
            );
            r.render(
                &self.device,
                &self.queue,
                &view_tex,
                self.target.is_some(),
                &frustum,
                eye,
            );
        }
        self.queue.present(surface_texture);

        self.frames += 1;
        if self.last_log.elapsed().as_secs_f64() >= 1.0 {
            let fps = self.frames as f64 / self.last_log.elapsed().as_secs_f64();
            let avg = self
                .stats
                .average_fps()
                .map(|v| format!("{v:.1}"))
                .unwrap_or_else(|| "n/a".into());
            let low = self
                .stats
                .low_01_fps()
                .map(|v| format!("{v:.1}"))
                .unwrap_or_else(|| "n/a".into());
            println!("FPS: {fps:.1} | avg: {avg} | 0.1% low: {low}");
            self.frames = 0;
            self.last_log = Instant::now();
        }

        // Benchmark completion: after the measured window, print a summary and
        // signal the event loop to exit.
        if let (Some(start), Some(secs)) = (self.bench_start, self.bench_secs) {
            if start.elapsed().as_secs_f32() >= secs {
                if let Some((avg, low)) = summarize(&self.bench_times) {
                    println!(
                        "BENCH_RESULT avg={avg:.1} 0.1%low={low:.1} frames={}",
                        self.bench_times.len()
                    );
                }
                self.bench_start = None;
                return true;
            }
        }
        false
    }
}

struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Sunset"))
                .unwrap(),
        );
        let mut state = pollster::block_on(State::new(window));
        // Capture the cursor so mouse-look works immediately.
        let _ = state.window.set_cursor_grab(CursorGrabMode::Locked);
        state.window.set_cursor_visible(false);
        state.cursor_captured = true;
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(state) = &mut self.state {
                    state.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(state) = &mut self.state {
                    if let PhysicalKey::Code(code) = event.physical_key {
                        if event.state == ElementState::Pressed {
                            state.keys.insert(code);
                            if code == KeyCode::Escape && state.cursor_captured {
                                state.cursor_captured = false;
                                let _ = state.window.set_cursor_grab(CursorGrabMode::None);
                                state.window.set_cursor_visible(true);
                            }
                            if let Some(slot) = digit_slot(code) {
                                state.intent.select_slot = Some(slot);
                            }
                            if code == KeyCode::F5 {
                                let json = save::to_json(&state.sim.world, &state.sim.player);
                                match save::write_atomic(Path::new(SAVE_PATH), &json) {
                                    Ok(()) => println!("Saved to {SAVE_PATH}"),
                                    Err(e) => eprintln!("Save failed: {e}"),
                                }
                            }
                            if code == KeyCode::F11 {
                                state.fullscreen = !state.fullscreen;
                                state.window.set_fullscreen(if state.fullscreen {
                                    Some(Fullscreen::Borderless(None))
                                } else {
                                    None
                                });
                            }
                        } else {
                            state.keys.remove(&code);
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                if let Some(state) = &mut self.state {
                    match button_state {
                        ElementState::Pressed => {
                            state.mouse_buttons.insert(button);
                            // Left-click re-captures the cursor when free.
                            if button == MouseButton::Left && !state.cursor_captured {
                                let _ = state.window.set_cursor_grab(CursorGrabMode::Locked);
                                state.window.set_cursor_visible(false);
                                state.cursor_captured = true;
                            }
                        }
                        ElementState::Released => {
                            state.mouse_buttons.remove(&button);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let Some(state) = &mut self.state {
            if let DeviceEvent::MouseMotion { delta } = event {
                if state.cursor_captured {
                    let half = std::f32::consts::FRAC_PI_2;
                    state.sim.player.pitch =
                        (state.sim.player.pitch - delta.1 as f32 * SENSITIVITY)
                            .clamp(-half + 0.001, half - 0.001);
                    state.sim.player.yaw -= delta.0 as f32 * SENSITIVITY;
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            if state.render() {
                event_loop.exit();
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("could not create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App { state: None };
    event_loop.run_app(&mut app).expect("event loop error");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frustum_culls_invisible_chunks() {
        let eye = Vec3::new(0.0, 0.0, 0.0);
        let view = glam::camera::rh::view::look_to_mat4(eye, Vec3::NEG_Z, Vec3::Y);
        let proj = glam::camera::rh::proj::directx::perspective(
            70.0_f32.to_radians(),
            1.0,
            0.1,
            1000.0,
        );
        let frustum = Frustum::from_matrix(&(proj * view));

        // In front, on-axis: visible.
        assert!(frustum.intersects_aabb([-8.0, 0.0, -108.0], [8.0, 64.0, -100.0]));
        // Behind the camera: culled.
        assert!(!frustum.intersects_aabb([-8.0, 0.0, 100.0], [8.0, 64.0, 108.0]));
        // Far to the side (outside the 70° FOV): culled.
        assert!(!frustum.intersects_aabb([1000.0, 0.0, -10.0], [1016.0, 64.0, 6.0]));
    }
}




