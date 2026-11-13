#![forbid(unsafe_code)]

mod allocator;
mod arena;
mod camera;
mod cpu_trace;
mod culling;
mod draw;
mod font;
mod gpu_timer;
mod hud;
mod present_mode;
mod shader;
mod shading;
mod sorting;
mod staging;
mod terrain_scene;

pub use allocator::{Range, RangeAllocator};
pub use arena::GeometryArena;
pub use camera::{look_at, perspective, view_projection, Mat4};
pub use cpu_trace::{CpuSpan, CpuTrace};
pub use culling::{Aabb, Frustum, Plane};
pub use draw::{
    build_indirect_commands, select_draw_mode, visible_sections, DrawCommand, DrawMode, SectionDraw,
};
pub use font::{glyph_rows, GLYPH_ADVANCE, GLYPH_HEIGHT, GLYPH_WIDTH};
pub use gpu_timer::GpuTimer;
pub use hud::{HudBatch, HudMaterial, HudVertex};
pub use present_mode::select_present_mode;
pub use shader::TERRAIN_SHADER;
pub use shading::{ao_factor, apply_tint, face_uv, fog_factor, light_brightness, shade_color, Fog};
pub use sorting::{sort_translucent_back_to_front, TranslucentSection};
pub use staging::{
    StagingRing, UploadBudget, DEFAULT_UPLOAD_BUDGET_BYTES, DEFAULT_UPLOAD_BUDGET_US,
};
pub use terrain_scene::{build_terrain_scene, SceneSection, TerrainScene};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    present_mode: wgpu::PresentMode,
    size: (u32, u32),
    adapter_name: String,
    gpu_timer: Option<GpuTimer>,
    cpu_trace: CpuTrace,
}

pub struct RendererInit {
    pub backends: wgpu::Backends,
    pub power_preference: wgpu::PowerPreference,
    pub width: u32,
    pub height: u32,
    pub timing: bool,
}

impl Default for RendererInit {
    fn default() -> Self {
        Self {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            power_preference: wgpu::PowerPreference::HighPerformance,
            width: 1920,
            height: 1080,
            timing: true,
        }
    }
}

pub enum FrameOutcome {
    Rendered,
    Suboptimal,
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
}

impl Renderer {
    pub fn new(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        init: &RendererInit,
    ) -> Result<Self, String> {
        let instance = Self::create_instance();
        let surface = instance
            .create_surface(target)
            .map_err(|error| format!("create surface: {error}"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: init.power_preference,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .map_err(|error| format!("request adapter: {error}"))?;
        let timestamp_supported =
            init.timing && adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: if timestamp_supported {
                wgpu::Features::TIMESTAMP_QUERY
            } else {
                wgpu::Features::empty()
            },
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        }))
        .map_err(|error| format!("request device: {error}"))?;
        let present_mode = select_present_mode(&surface.get_capabilities(&adapter).present_modes);
        let config = surface
            .get_default_config(&adapter, init.width, init.height)
            .ok_or_else(|| "surface is not supported by the adapter".to_string())?;
        let info = adapter.get_info();
        let adapter_name = format!("{} ({:?}, driver {})", info.name, info.backend, info.driver);
        let gpu_timer = if timestamp_supported {
            Some(GpuTimer::new(&device, &queue))
        } else {
            None
        };
        let mut renderer = Self {
            surface,
            device,
            queue,
            config,
            present_mode,
            size: (init.width, init.height),
            adapter_name,
            gpu_timer,
            cpu_trace: CpuTrace::default(),
        };
        renderer.reconfigure();
        Ok(renderer)
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub fn present_mode(&self) -> wgpu::PresentMode {
        self.present_mode
    }

    pub fn present_mode_label(&self) -> &'static str {
        match self.present_mode {
            wgpu::PresentMode::Immediate => "Immediate",
            wgpu::PresentMode::Mailbox => "Mailbox",
            wgpu::PresentMode::Fifo => "Fifo",
            wgpu::PresentMode::FifoRelaxed => "FifoRelaxed",
            wgpu::PresentMode::AutoVsync => "AutoVsync",
            wgpu::PresentMode::AutoNoVsync => "AutoNoVsync",
        }
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn gpu_pass_ms(&self) -> f32 {
        self.gpu_timer
            .as_ref()
            .map(GpuTimer::duration_ms)
            .unwrap_or(0.0)
    }

    pub fn readback_gpu_timing(&mut self) {
        if let Some(timer) = self.gpu_timer.as_mut() {
            let device = &self.device;
            timer.readback_latest(device);
        }
    }

    pub fn cpu_span_us(&self, span: CpuSpan) -> f64 {
        self.cpu_trace.frame_us(span)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.size = (width, height);
        self.reconfigure();
    }

    pub fn render(&mut self) -> FrameOutcome {
        self.cpu_trace.reset();

        self.cpu_trace.begin(CpuSpan::AcquirePresent);
        let (frame, outcome) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, FrameOutcome::Rendered),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, FrameOutcome::Suboptimal),
            wgpu::CurrentSurfaceTexture::Timeout => {
                self.cpu_trace.end();
                return FrameOutcome::Timeout;
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                self.cpu_trace.end();
                return FrameOutcome::Occluded;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.cpu_trace.end();
                return FrameOutcome::Outdated;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.cpu_trace.end();
                return FrameOutcome::Lost;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                self.cpu_trace.end();
                return FrameOutcome::Validation;
            }
        };
        self.cpu_trace.end();

        self.cpu_trace.begin(CpuSpan::CommandEncode);
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let timestamp_writes = self.gpu_timer.as_ref().map(GpuTimer::timestamp_writes);
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        if let Some(timer) = &mut self.gpu_timer {
            timer.resolve(&mut encoder);
        }
        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);
        self.cpu_trace.end();

        self.cpu_trace.begin(CpuSpan::AcquirePresent);
        self.queue.present(frame);
        self.cpu_trace.end();

        if let Some(timer) = &mut self.gpu_timer {
            timer.advance();
        }
        outcome
    }

    fn create_instance() -> wgpu::Instance {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::VULKAN | wgpu::Backends::GL;
        descriptor.flags = if cfg!(debug_assertions) {
            wgpu::InstanceFlags::debugging()
        } else {
            wgpu::InstanceFlags::empty()
        };
        wgpu::Instance::new(descriptor)
    }

    pub fn reconfigure(&mut self) {
        self.config.width = self.size.0;
        self.config.height = self.size.1;
        self.config.present_mode = self.present_mode;
        self.config.desired_maximum_frame_latency = 2;
        self.surface.configure(&self.device, &self.config);
    }
}
