use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use lumen_core::Color;
use lumen_render::{DrawCommand, GlyphUpload, Renderer, TextBatch, TextGlyph};
use lumen_text::{TextBuffer, TextSystem};
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

// ---------------------------------------------------------------------------
// TextContext — bridges lumen-text and lumen-render
// ---------------------------------------------------------------------------

/// Bridges text shaping (lumen-text) and GPU rendering (lumen-render).
pub struct TextContext {
    system: TextSystem,
    uploaded_keys: HashSet<u64>,
}

impl TextContext {
    /// Create a new text context.
    pub fn new() -> Self {
        Self {
            system: TextSystem::new(),
            uploaded_keys: HashSet::new(),
        }
    }

    /// Create a shaped text buffer.
    pub fn create_buffer(
        &mut self,
        text: &str,
        font_size: f32,
        line_height: f32,
        max_width: Option<f32>,
    ) -> TextBuffer {
        self.system
            .create_buffer(text, font_size, line_height, max_width)
    }

    /// Resize a text buffer's wrapping width.
    pub fn resize_buffer(&mut self, buffer: &mut TextBuffer, max_width: Option<f32>) {
        self.system.resize_buffer(buffer, max_width);
    }

    /// Produce a draw command for a text buffer at the given position.
    pub fn draw_text(&mut self, buffer: &TextBuffer, x: f32, y: f32, color: Color) -> DrawCommand {
        let shaped = self.system.shaped_glyphs(buffer);
        let mut uploads = Vec::new();
        let mut glyphs = Vec::new();

        for g in &shaped {
            if !self.uploaded_keys.contains(&g.cache_key) {
                uploads.push(GlyphUpload {
                    cache_key: g.cache_key,
                    width: g.width,
                    height: g.height,
                    data: g.data.clone(),
                });
                self.uploaded_keys.insert(g.cache_key);
            }
            glyphs.push(TextGlyph {
                x: x + g.x,
                y: y + g.y,
                width: g.width,
                height: g.height,
                cache_key: g.cache_key,
                color,
            });
        }

        DrawCommand::Text(TextBatch { uploads, glyphs })
    }
}

impl Default for TextContext {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// DesktopApp — plain draw_fn (no text)
// ---------------------------------------------------------------------------

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
        let (window, gpu) = init_gpu(event_loop);
        self.gpu = Some(gpu);
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

                let gpu = self.gpu.as_mut().expect("gpu state present");
                let window = self.window.as_ref().expect("window present");

                render_frame(gpu, &commands, [logical_w, logical_h]);
                update_fps(window, &mut self.frame_count, &mut self.fps_instant);
                window.request_redraw();
            }

            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// DesktopTextApp — draw_fn receives &mut TextContext
// ---------------------------------------------------------------------------

type TextDrawFn = Box<dyn FnMut(f32, f32, &mut TextContext) -> Vec<DrawCommand>>;

struct DesktopTextApp {
    draw_fn: TextDrawFn,
    text_ctx: TextContext,
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    frame_count: u32,
    fps_instant: Instant,
}

impl DesktopTextApp {
    fn new(draw_fn: TextDrawFn) -> Self {
        Self {
            draw_fn,
            text_ctx: TextContext::new(),
            window: None,
            gpu: None,
            frame_count: 0,
            fps_instant: Instant::now(),
        }
    }
}

impl ApplicationHandler for DesktopTextApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let (window, gpu) = init_gpu(event_loop);
        self.gpu = Some(gpu);
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

                let commands = (self.draw_fn)(logical_w, logical_h, &mut self.text_ctx);

                let gpu = self.gpu.as_mut().expect("gpu state present");
                let window = self.window.as_ref().expect("window present");

                render_frame(gpu, &commands, [logical_w, logical_h]);
                update_fps(window, &mut self.frame_count, &mut self.fps_instant);
                window.request_redraw();
            }

            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn init_gpu(event_loop: &ActiveEventLoop) -> (Arc<Window>, GpuState) {
    let attrs = WindowAttributes::default()
        .with_title("Lumen")
        .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));

    let window = Arc::new(
        event_loop
            .create_window(attrs)
            .expect("window creation failed"),
    );

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

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

    let gpu = GpuState {
        surface,
        device,
        queue,
        config,
        renderer,
    };

    (window, gpu)
}

fn render_frame(gpu: &mut GpuState, commands: &[DrawCommand], viewport: [f32; 2]) {
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
    gpu.renderer
        .render(&gpu.device, &gpu.queue, &view, viewport, commands);
    frame.present();
}

fn update_fps(window: &Window, frame_count: &mut u32, fps_instant: &mut Instant) {
    *frame_count += 1;
    let elapsed = fps_instant.elapsed();
    if elapsed.as_secs_f64() >= 1.0 {
        let fps = *frame_count as f64 / elapsed.as_secs_f64();
        window.set_title(&format!("Lumen — {fps:.0} FPS"));
        *frame_count = 0;
        *fps_instant = Instant::now();
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run a desktop app. `draw_fn` is called each frame with `(width, height)` in logical pixels.
pub fn run(draw_fn: impl FnMut(f32, f32) -> Vec<DrawCommand> + 'static) {
    let event_loop = EventLoop::new().expect("event loop creation failed");
    let mut app = DesktopApp::new(Box::new(draw_fn));
    event_loop.run_app(&mut app).expect("event loop error");
}

/// Run a desktop app with text support. `draw_fn` receives a [`TextContext`] for text rendering.
pub fn run_with_text(
    draw_fn: impl FnMut(f32, f32, &mut TextContext) -> Vec<DrawCommand> + 'static,
) {
    let event_loop = EventLoop::new().expect("event loop creation failed");
    let mut app = DesktopTextApp::new(Box::new(draw_fn));
    event_loop.run_app(&mut app).expect("event loop error");
}
