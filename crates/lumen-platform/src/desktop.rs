use std::sync::Arc;
use std::time::Instant;

use lumen_render::{DrawCommand, Renderer};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
}

struct DesktopApp {
    draw_fn: Box<dyn FnMut(f32, f32) -> Vec<DrawCommand>>,
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    frame_count: u32,
    fps_instant: Instant,
}

impl DesktopApp {
    fn new(draw_fn: Box<dyn FnMut(f32, f32) -> Vec<DrawCommand>>) -> Self {
        Self {
            draw_fn,
            window: None,
            gpu: None,
            frame_count: 0,
            fps_instant: Instant::now(),
        }
    }
}

impl ApplicationHandler for DesktopApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Lumen")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));

        // Window creation is required for the app to function
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("window creation failed"),
        );

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        // GPU surface, adapter, and device are required for rendering
        let surface = instance
            .create_surface(window.clone())
            .expect("surface creation failed");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("no compatible GPU adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("GPU device creation failed");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, format);

        self.gpu = Some(GpuState {
            surface,
            device,
            queue,
            config,
            renderer,
        });
        self.window = Some(window.clone());
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(new_size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.config.width = new_size.width.max(1);
                    gpu.config.height = new_size.height.max(1);
                    gpu.surface.configure(&gpu.device, &gpu.config);
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                let (logical_w, logical_h) = match (&self.window, &self.gpu) {
                    (Some(w), Some(g)) => {
                        let s = w.scale_factor() as f32;
                        (g.config.width as f32 / s, g.config.height as f32 / s)
                    }
                    _ => return,
                };

                let commands = (self.draw_fn)(logical_w, logical_h);

                let gpu = self
                    .gpu
                    .as_ref()
                    .expect("gpu state missing after size check");
                let window = self
                    .window
                    .as_ref()
                    .expect("window missing after size check");

                let frame = match gpu.surface.get_current_texture() {
                    Ok(f) => f,
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        gpu.surface.configure(&gpu.device, &gpu.config);
                        return;
                    }
                    Err(e) => {
                        eprintln!("surface error: {e}");
                        return;
                    }
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                gpu.renderer.render(
                    &gpu.device,
                    &gpu.queue,
                    &view,
                    [logical_w, logical_h],
                    &commands,
                );
                frame.present();

                self.frame_count += 1;
                let elapsed = self.fps_instant.elapsed();
                if elapsed.as_secs_f64() >= 1.0 {
                    let fps = self.frame_count as f64 / elapsed.as_secs_f64();
                    window.set_title(&format!("Lumen — {fps:.0} FPS"));
                    self.frame_count = 0;
                    self.fps_instant = Instant::now();
                }

                window.request_redraw();
            }

            _ => {}
        }
    }
}

/// Run a desktop app. `draw_fn` is called each frame with `(width, height)` in logical pixels.
pub fn run(draw_fn: impl FnMut(f32, f32) -> Vec<DrawCommand> + 'static) {
    let event_loop = EventLoop::new().expect("event loop creation failed");
    let mut app = DesktopApp::new(Box::new(draw_fn));
    event_loop.run_app(&mut app).expect("event loop error");
}
