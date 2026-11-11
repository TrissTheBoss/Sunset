#![forbid(unsafe_code)]

mod present_mode;

pub use present_mode::select_present_mode;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    present_mode: wgpu::PresentMode,
    size: (u32, u32),
    adapter_name: String,
}

pub struct RendererInit {
    pub backends: wgpu::Backends,
    pub power_preference: wgpu::PowerPreference,
    pub width: u32,
    pub height: u32,
}

impl Default for RendererInit {
    fn default() -> Self {
        Self {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            power_preference: wgpu::PowerPreference::HighPerformance,
            width: 1920,
            height: 1080,
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
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
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
        let mut renderer = Self {
            surface,
            device,
            queue,
            config,
            present_mode,
            size: (init.width, init.height),
            adapter_name,
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

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.size = (width, height);
        self.reconfigure();
    }

    pub fn render(&mut self) -> FrameOutcome {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                self.present(frame);
                return FrameOutcome::Suboptimal;
            }
            wgpu::CurrentSurfaceTexture::Timeout => return FrameOutcome::Timeout,
            wgpu::CurrentSurfaceTexture::Occluded => return FrameOutcome::Occluded,
            wgpu::CurrentSurfaceTexture::Outdated => return FrameOutcome::Outdated,
            wgpu::CurrentSurfaceTexture::Lost => return FrameOutcome::Lost,
            wgpu::CurrentSurfaceTexture::Validation => return FrameOutcome::Validation,
        };
        self.present(frame);
        FrameOutcome::Rendered
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

    fn present(&mut self, frame: wgpu::SurfaceTexture) {
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
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
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);
        self.queue.present(frame);
    }
}
