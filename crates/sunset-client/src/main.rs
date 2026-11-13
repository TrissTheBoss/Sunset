#![forbid(unsafe_code)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sunset_metrics::IntervalRing;
use sunset_render::{
    build_terrain_scene, view_projection, CameraUniform, CpuSpan, FrameOutcome, Renderer,
    RendererInit,
};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

const WARMUP_DEFAULT_S: f64 = 2.0;
const SAMPLING_DEFAULT_S: f64 = 5.0;
const INTERVAL_CAPACITY: usize = 1_048_576;

#[derive(Deserialize)]
struct SceneManifest {
    scene: SceneName,
    #[serde(default)]
    timing: Timing,
}

#[derive(Deserialize)]
struct SceneName {
    name: String,
}

#[derive(Deserialize, Default)]
struct Timing {
    #[serde(default)]
    warmup_s: f64,
    #[serde(default)]
    sampling_s: f64,
}

struct RunConfig {
    name: String,
    warmup: Duration,
    sampling: Duration,
}

#[derive(Serialize)]
struct Metrics {
    schema: u32,
    scene: String,
    frames: u64,
    elapsed_s: f64,
    present_mode: String,
    intervals_ms: Vec<f32>,
    gpu_pass_ms: f32,
    cpu_command_encode_us: f64,
    cpu_acquire_present_us: f64,
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    run: RunConfig,
    run_start: Instant,
    last_frame: Instant,
    intervals: IntervalRing,
    frames: u64,
}

impl App {
    fn new(run: RunConfig) -> Self {
        Self {
            window: None,
            renderer: None,
            run,
            run_start: Instant::now(),
            last_frame: Instant::now(),
            intervals: IntervalRing::new(INTERVAL_CAPACITY),
            frames: 0,
        }
    }

    fn finish(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(renderer) = &mut self.renderer {
            renderer.readback_gpu_timing();
            let metrics = Metrics {
                schema: 1,
                scene: self.run.name.clone(),
                frames: self.frames,
                elapsed_s: self.run.sampling.as_secs_f64(),
                present_mode: renderer.present_mode_label().to_string(),
                intervals_ms: self.intervals.values(),
                gpu_pass_ms: renderer.gpu_pass_ms(),
                cpu_command_encode_us: renderer.cpu_span_us(CpuSpan::CommandEncode),
                cpu_acquire_present_us: renderer.cpu_span_us(CpuSpan::AcquirePresent),
            };
            if let Ok(json) = serde_json::to_string(&metrics) {
                println!("{json}");
            }
        }
        event_loop.exit();
    }

    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let since_start = now.duration_since(self.run_start);
        if since_start >= self.run.warmup {
            let interval_ms = now.duration_since(self.last_frame).as_secs_f64() * 1000.0;
            self.intervals.push(interval_ms as f32);
            self.frames += 1;
        }
        self.last_frame = now;
        if since_start >= self.run.warmup + self.run.sampling {
            self.finish(event_loop);
            return;
        }
        if let Some(renderer) = &mut self.renderer {
            match renderer.render() {
                FrameOutcome::Suboptimal | FrameOutcome::Outdated | FrameOutcome::Lost => {
                    renderer.reconfigure();
                }
                FrameOutcome::Validation => {
                    eprintln!("surface validation error");
                }
                _ => {}
            }
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = WindowAttributes::default()
            .with_title("Sunset")
            .with_inner_size(LogicalSize::new(1920.0, 1080.0));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("create window: {error}");
                event_loop.exit();
                return;
            }
        };
        let size = window.inner_size();
        let timing = std::env::var("SUNSET_GPU_TIMING")
            .map(|value| value != "0")
            .unwrap_or(true);
        let init = RendererInit {
            width: size.width,
            height: size.height,
            timing,
            ..Default::default()
        };
        match Renderer::new(Arc::clone(&window), &init) {
            Ok(mut renderer) => {
                eprintln!(
                    "adapter: {}; present mode: {}",
                    renderer.adapter_name(),
                    renderer.present_mode_label()
                );
                let coords: Vec<(i32, i32, i32)> = (-1..=1)
                    .flat_map(|sx| (-1..=1).flat_map(move |sz| (2..=5).map(move |sy| (sx, sy, sz))))
                    .collect();
                let scene = build_terrain_scene(12345, &coords);
                let vp = view_projection(
                    [8.0, 100.0, 40.0],
                    [8.0, 70.0, 0.0],
                    [0.0, 1.0, 0.0],
                    (70.0f32).to_radians(),
                    16.0 / 9.0,
                    0.1,
                    300.0,
                );
                let camera = CameraUniform::new(vp, 10.0, 250.0, [0.7, 0.8, 0.9], [1.0, 1.0, 1.0]);
                let terrain = renderer.create_terrain(&scene, &camera);
                renderer.set_terrain(terrain);
                self.window = Some(window);
                self.renderer = Some(renderer);
                self.run_start = Instant::now();
                self.last_frame = Instant::now();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            Err(error) => {
                eprintln!("create renderer: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let matches = self
            .window
            .as_ref()
            .map(|window| window.id() == window_id)
            .unwrap_or(false);
        if !matches {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.frame(event_loop),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn parse_run(args: &[String]) -> RunConfig {
    let manifest = args
        .get(1)
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| toml::from_str::<SceneManifest>(&text).ok());
    match manifest {
        Some(manifest) => RunConfig {
            name: manifest.scene.name,
            warmup: Duration::from_secs_f64(manifest.timing.warmup_s.max(0.0)),
            sampling: Duration::from_secs_f64(manifest.timing.sampling_s.max(0.0)),
        },
        None => RunConfig {
            name: "unknown".to_string(),
            warmup: Duration::from_secs_f64(WARMUP_DEFAULT_S),
            sampling: Duration::from_secs_f64(SAMPLING_DEFAULT_S),
        },
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let run = parse_run(&args);
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(error) => {
            eprintln!("event loop: {error}");
            std::process::exit(2);
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(run);
    if let Err(error) = event_loop.run_app(&mut app) {
        eprintln!("event loop: {error}");
        std::process::exit(3);
    }
}
