use crate::camera::Mat4;
use crate::shader::TERRAIN_SHADER;
use crate::terrain_scene::TerrainScene;
use std::borrow::Cow;

const BLOCK_COLORS: [[u8; 4]; 9] = [
    [0, 0, 0, 0],
    [128, 128, 128, 255],
    [134, 96, 67, 255],
    [95, 159, 53, 255],
    [219, 207, 163, 255],
    [63, 118, 228, 160],
    [60, 110, 40, 255],
    [102, 81, 50, 255],
    [240, 240, 240, 255],
];

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CameraUniform {
    view_proj: [f32; 16],
    fog_start: f32,
    fog_end: f32,
    pad: [f32; 2],
    fog_color: [f32; 4],
    tint_color: [f32; 4],
}

impl CameraUniform {
    pub fn new(
        view_proj: Mat4,
        fog_start: f32,
        fog_end: f32,
        fog_color: [f32; 3],
        tint_color: [f32; 3],
    ) -> Self {
        Self {
            view_proj: view_proj.as_array(),
            fog_start,
            fog_end,
            pad: [0.0; 2],
            fog_color: [fog_color[0], fog_color[1], fog_color[2], 1.0],
            tint_color: [tint_color[0], tint_color[1], tint_color[2], 1.0],
        }
    }

    fn to_bytes(self) -> [u8; 112] {
        let mut out = [0u8; 112];
        let mut offset = 0;
        for value in &self.view_proj {
            out[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            offset += 4;
        }
        out[offset..offset + 4].copy_from_slice(&self.fog_start.to_le_bytes());
        offset += 4;
        out[offset..offset + 4].copy_from_slice(&self.fog_end.to_le_bytes());
        offset += 4;
        offset += 8;
        for value in &self.fog_color {
            out[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            offset += 4;
        }
        for value in &self.tint_color {
            out[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            offset += 4;
        }
        out
    }
}

#[derive(Clone, Copy)]
struct DrawRange {
    first_index: u32,
    index_count: u32,
    base_vertex: i32,
}

pub struct TerrainRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    draws: Vec<DrawRange>,
}

impl TerrainRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
        scene: &TerrainScene,
        camera: &CameraUniform,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(TERRAIN_SHADER)),
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: scene.vertices.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buffer, 0, &scene.vertices);

        let index_bytes: Vec<u8> = scene
            .indices
            .iter()
            .flat_map(|index| index.to_le_bytes())
            .collect();
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: index_bytes.len() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&index_buffer, 0, &index_bytes);

        let mut pixels = Vec::with_capacity(BLOCK_COLORS.len() * 4);
        for color in BLOCK_COLORS {
            pixels.extend_from_slice(&color);
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: BLOCK_COLORS.len() as u32,
            },
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
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: BLOCK_COLORS.len() as u32,
            },
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let camera_bytes = camera.to_bytes();
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: camera_bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform_buffer, 0, &camera_bytes);

        let origins: Vec<[f32; 4]> = scene
            .sections
            .iter()
            .map(|section| [section.origin[0], section.origin[1], section.origin[2], 0.0])
            .collect();
        let origin_bytes: Vec<u8> = origins
            .iter()
            .flat_map(|origin| origin.iter().flat_map(|v| v.to_le_bytes()))
            .collect();
        let origin_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: origin_bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&origin_buffer, 0, &origin_bytes);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
                        view_dimension: wgpu::TextureViewDimension::D2Array,
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 8,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Uint32x2],
                })],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_opaque"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: origin_buffer.as_entire_binding(),
                },
            ],
        });

        let draws = scene
            .sections
            .iter()
            .map(|section| DrawRange {
                first_index: section.first_index,
                index_count: section.index_count,
                base_vertex: section.base_vertex,
            })
            .collect();

        Self {
            pipeline,
            bind_group,
            vertex_buffer,
            index_buffer,
            draws,
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        for (section_index, draw) in self.draws.iter().enumerate() {
            let instance = section_index as u32;
            pass.draw_indexed(
                draw.first_index..draw.first_index + draw.index_count,
                draw.base_vertex,
                instance..instance + 1,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn camera_uniform_layout_matches_wgsl() {
        assert_eq!(size_of::<CameraUniform>(), 112);
        assert_eq!(offset_of!(CameraUniform, view_proj), 0);
        assert_eq!(offset_of!(CameraUniform, fog_start), 64);
        assert_eq!(offset_of!(CameraUniform, fog_end), 68);
        assert_eq!(offset_of!(CameraUniform, fog_color), 80);
        assert_eq!(offset_of!(CameraUniform, tint_color), 96);
        assert_eq!(
            CameraUniform::new(Mat4::IDENTITY, 0.0, 100.0, [0.5, 0.5, 0.5], [1.0, 1.0, 1.0])
                .to_bytes()
                .len(),
            112
        );
    }
}
