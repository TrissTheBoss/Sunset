//! M1 — a minimal raw `winit` + `wgpu` renderer that draws the world's opaque
//! greedy mesh (atlas-textured) through an unlit pipeline with a depth buffer.
//! See RAW_RENDERER_PLAN.md.

mod assets;
mod mesh;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Quat, Vec2, Vec3};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use sunset_core::block::{block_def, BlockState};
use sunset_core::fixture;
use sunset_core::intent::{BlockAction, PlayerIntent};
use sunset_core::physics;
use sunset_core::player::{Player, PLAYER_EYE_HEIGHT, PLAYER_HEIGHT};
use sunset_core::simulation::Simulation;
use sunset_core::world::World;

use assets::FaceTextures;
use mesh::{MeshData, Vertex};

/// Look sensitivity in radians per pixel of mouse motion.
const SENSITIVITY: f32 = 0.0025;
/// Fixed simulation tick (20 Hz), matching the Bevy adapter.
const FIXED_DT: f32 = 1.0 / 20.0;
/// Block interaction reach distance (blocks).
const REACH: f32 = 5.0;

/// Per-frame camera data: the world view-projection plus the outline's model
/// matrix (the outline is a static unit cube scaled/translated per frame).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    outline_model: [[f32; 4]; 4],
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
            format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
            offset: 20,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: 36,
            shader_location: 3,
            format: wgpu::VertexFormat::Float32x4,
        },
    ],
};

const SHADER: &str = r#"
struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) rect: vec4<f32>,
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
    out.uv = in.uv;
    out.color = in.color;
    out.rect = in.rect;
    return out;
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

/// One drawable mesh: a pipeline plus its vertex/index buffers.
struct Drawable {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

/// Everything needed to draw the world's opaque and transparent meshes.
struct Renderer {
    opaque: Drawable,
    transparent: Drawable,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    depth_view: wgpu::TextureView,
    line_pipeline: wgpu::RenderPipeline,
    line_vertex_buffer: wgpu::Buffer,
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
        opaque: &MeshData,
        transparent: &MeshData,
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

        let opaque_drawable = create_drawable(
            device,
            queue,
            &shader,
            &pipeline_layout,
            format,
            None,
            true,
            opaque,
        );
        let transparent_drawable = create_drawable(
            device,
            queue,
            &shader,
            &pipeline_layout,
            format,
            Some(wgpu::BlendState::ALPHA_BLENDING),
            false,
            transparent,
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

        Self {
            opaque: opaque_drawable,
            transparent: transparent_drawable,
            bind_group,
            uniform_buffer,
            depth_view,
            line_pipeline,
            line_vertex_buffer,
        }
    }

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.depth_view = create_depth_view(device, width, height);
    }

    fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, outline: bool) {
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
            self.draw(&mut pass, &self.opaque);
        }

        // Transparent pass: loads the opaque color/depth, alpha-blends, and
        // tests depth without writing it (so glass is correctly occluded and
        // occludes, but doesn't punch holes in the depth buffer).
        if self.transparent.index_count > 0 {
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
            self.draw(&mut pass, &self.transparent);
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

    /// Re-upload the opaque and transparent meshes after a block edit. The
    /// pipelines are reused; only the vertex/index buffers are recreated.
    fn rebuild_meshes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        opaque: &MeshData,
        transparent: &MeshData,
    ) {
        let (vb, ib, n) = create_mesh_buffers(device, queue, opaque);
        self.opaque.vertex_buffer = vb;
        self.opaque.index_buffer = ib;
        self.opaque.index_count = n;
        let (vb, ib, n) = create_mesh_buffers(device, queue, transparent);
        self.transparent.vertex_buffer = vb;
        self.transparent.index_buffer = ib;
        self.transparent.index_count = n;
    }

    fn draw(&self, pass: &mut wgpu::RenderPass<'_>, drawable: &Drawable) {
        pass.set_pipeline(&drawable.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, drawable.vertex_buffer.slice(..));
        pass.set_index_buffer(drawable.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..drawable.index_count, 0, 0..1);
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

/// Build a mesh's render pipeline (buffers are created separately so they can
/// be rebuilt after an edit without recreating the pipeline).
///
/// `blend` selects alpha blending (transparent) vs. `None` (opaque), and
/// `depth_write` controls whether the pass writes to the depth buffer.
#[allow(clippy::too_many_arguments)]
fn create_drawable(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shader: &wgpu::ShaderModule,
    pipeline_layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
    depth_write: bool,
    mesh: &MeshData,
) -> Drawable {
    let (vertex_buffer, index_buffer, index_count) = create_mesh_buffers(device, queue, mesh);

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
    });

    Drawable {
        pipeline,
        vertex_buffer,
        index_buffer,
        index_count,
    }
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

/// Mesh every chunk in the world into combined opaque + transparent meshes.
fn mesh_world(
    world: &World,
    face_textures: &HashMap<BlockState, FaceTextures>,
    uv: &HashMap<String, [f32; 4]>,
) -> (MeshData, MeshData) {
    let mut opaque = MeshData::new();
    let mut transparent = MeshData::new();
    for chunk in world.chunks() {
        let (o, t) = mesh::build_chunk_meshes(world, chunk, face_textures, uv);
        let obase = opaque.vertices.len() as u32;
        opaque.vertices.extend(o.vertices);
        opaque.indices.extend(o.indices.iter().map(|i| obase + i));
        let tbase = transparent.vertices.len() as u32;
        transparent.vertices.extend(t.vertices);
        transparent.indices.extend(t.indices.iter().map(|i| tbase + i));
    }
    (opaque, transparent)
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
    /// Sum of chunk revisions, used to detect edits that need a re-mesh.
    world_revision: u64,
    last_time: Instant,
    accumulator: f32,
    frames: u64,
    last_log: Instant,
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
        let world = fixture::building_yard();
        let (opaque, transparent) = mesh_world(&world, &face_textures, &uv);
        println!(
            "opaque mesh: {} vertices, {} triangles",
            opaque.vertices.len(),
            opaque.indices.len() / 3
        );
        println!(
            "transparent mesh: {} vertices, {} triangles",
            transparent.vertices.len(),
            transparent.indices.len() / 3
        );

        let renderer = Renderer::new(
            &device,
            &queue,
            config.width,
            config.height,
            config.format,
            &atlas,
            &opaque,
            &transparent,
        );

        // Authoritative simulation, same spawn as the Bevy adapter.
        let player = Player::new(Vec3::new(8.0, 5.5, 16.0), 0.5, -0.25);
        let sim = Simulation::new(world, player);
        let world_revision: u64 = sim.world.chunks().map(|c| c.revision).sum();

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
            world_revision,
            last_time: Instant::now(),
            accumulator: 0.0,
            frames: 0,
            last_log: Instant::now(),
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

    fn rebuild_mesh(&mut self) {
        let (opaque, transparent) = mesh_world(&self.sim.world, &self.face_textures, &self.uv);
        if let Some(r) = self.renderer.as_mut() {
            r.rebuild_meshes(&self.device, &self.queue, &opaque, &transparent);
        }
    }

    fn render(&mut self) {
        // Step the authoritative simulation at a fixed 20 Hz.
        let now = Instant::now();
        self.accumulator += (now - self.last_time).as_secs_f32().min(0.1);
        self.last_time = now;
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

        // Re-mesh if any chunk revision changed (a block edit landed).
        let revision: u64 = self.sim.world.chunks().map(|c| c.revision).sum();
        if revision != self.world_revision {
            self.world_revision = revision;
            self.rebuild_mesh();
        }

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
        let uniform = CameraUniform {
            view_proj: (proj * view).to_cols_array_2d(),
            outline_model: match self.target {
                Some((pos, _face)) => outline_model_matrix(&self.sim.world, pos)
                    .unwrap_or(Mat4::IDENTITY.to_cols_array_2d()),
                None => Mat4::IDENTITY.to_cols_array_2d(),
            },
        };

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            _ => {
                self.surface.configure(&self.device, &self.config);
                return;
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
            r.render(&self.device, &self.queue, &view_tex, self.target.is_some());
        }
        self.queue.present(surface_texture);

        self.frames += 1;
        if self.last_log.elapsed().as_secs_f64() >= 1.0 {
            let fps = self.frames as f64 / self.last_log.elapsed().as_secs_f64();
            println!("FPS: {fps:.1}");
            self.frames = 0;
            self.last_log = Instant::now();
        }
    }
}

struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Sunset Raw — M4"))
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            state.render();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("could not create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App { state: None };
    event_loop.run_app(&mut app).expect("event loop error");
}




