use assets::Texture;
use glyphon::{Attrs, Cache, FontSystem, Metrics, SwashCache, TextArea, TextAtlas, TextBounds};
use imgui::{Condition, MouseCursor};
use imgui_winit_support::WinitPlatform;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;
use wgpu::{AdapterInfo, MultisampleState};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::util::ext::ColorExtensions;

pub mod assets;
pub mod entity;
pub mod world;

mod modules;

struct ImguiRenderer {
    context: imgui::Context,
    renderer: imgui_wgpu::Renderer,
    platform: WinitPlatform,
    clear_color: wgpu::Color,
    demo_open: bool,
    last_frame: Instant,
    last_cursor: Option<MouseCursor>,
}

pub struct TextRenderer<'a> {
    physical_size: PhysicalSize<u32>,
    scale_factor: f32,
    font_system: FontSystem,
    base_font: Attrs<'a>,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    atlas: TextAtlas,
    renderer: glyphon::TextRenderer,
    buffers: HashMap<String, glyphon::Buffer>,
}

pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: Option<wgpu::RenderPipeline>,

    pub adapter_info: AdapterInfo,

    // renderers
    text_renderer: TextRenderer<'a>,
    imgui_renderer: Option<ImguiRenderer>,

    loaded_textures: HashMap<String, assets::Texture>,
    texture_id: Option<String>,

    // temp
    rng: rand::rngs::ThreadRng,
    current_color: wgpu::Color,
    target_color: wgpu::Color,
    transition_speed: f32,

    last_frame_time: Option<Instant>,
    delta_time: Duration,
}

struct FrameContext {
    frame: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

impl<'a> Renderer<'a> {
    pub fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        // choose gpu
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        // show gpu info
        let info = adapter.get_info();
        println!(
            "{} on {} {} with {}",
            info.name, info.driver, info.driver_info, info.backend
        );

        // connect to gpu
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::default(),
        }))
        .unwrap();

        // create surface configuration
        let size = window.clone().inner_size();
        println!("window size is {} x {}", size.width, size.height);

        // kleur formaten
        let color_mode = glyphon::ColorMode::Accurate;
        let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;

        // maak en zet surface config met format en mailbox present mode
        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        config.format = swapchain_format;
        config.present_mode = wgpu::PresentMode::Mailbox;

        surface.configure(&device, &config);

        // tekst renderer
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let mut atlas =
            TextAtlas::with_color_mode(&device, &queue, &cache, swapchain_format, color_mode);
        let text_renderer =
            glyphon::TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);

        // maak font
        let font = Attrs::new()
            .family(glyphon::Family::SansSerif)
            .weight(glyphon::Weight::NORMAL);

        // zet scaling properties
        let scale_factor = window.clone().scale_factor() as f32;

        // kleuren randomizer
        let mut rng = rand::rng();
        let target = wgpu::Color::random(&mut rng);
        let current = wgpu::Color::random(&mut rng);

        println!("{:?}", current);
        println!("{:?}", target);

        let mut renderer = Renderer {
            surface: surface,
            device: device,
            queue: queue,
            config: config,
            window: window,
            render_pipeline: None,

            adapter_info: adapter.get_info(),

            imgui_renderer: None,
            text_renderer: TextRenderer {
                physical_size: size,
                scale_factor: scale_factor,
                font_system: font_system,
                base_font: font,
                swash_cache: swash_cache,
                viewport: viewport,
                atlas: atlas,
                renderer: text_renderer,
                buffers: HashMap::new(),
            },

            loaded_textures: HashMap::new(),
            texture_id: None,

            rng: rng,
            current_color: current,
            target_color: target,
            transition_speed: 0.01,

            last_frame_time: None,
            delta_time: Duration::from_secs_f32(0.0),
        };

        renderer.create_pipeline();
        renderer.create_imgui();
        renderer
    }

    pub fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        if size.height == 0 || size.width == 0 {
            return; // stop text adjustment if window size invalid
        }

        // adjust surface config width and height
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.window.request_redraw();

        // adjust text renderer's viewport to new surface config
        self.text_renderer.viewport.update(
            &self.queue,
            glyphon::Resolution {
                width: self.config.width,
                height: self.config.height,
            },
        );

        // adjust the text renderer's manual scale and size
        self.text_renderer.scale_factor = self.window.scale_factor() as f32;
        self.text_renderer.physical_size = size.cast();

        let logical_width = size.width as f32 / self.text_renderer.scale_factor;

        // idk wtf
        for (_, b) in self.text_renderer.buffers.iter_mut() {
            b.set_size(
                &mut self.text_renderer.font_system,
                Some(logical_width - 20.0),
                None,
            );
            b.shape_until_scroll(&mut self.text_renderer.font_system, false);
        }
    }

    pub fn handle_redraw(&mut self) -> Option<()> {
        let mut context = self.begin_frame()?;
        let dt_seconds = self.delta_time.as_secs_f32();

        self.display_imgui(&mut context, dt_seconds);
        self.display_text(&mut context, dt_seconds);

        self.end_frame(context);

        Some(())
    }

    pub fn load_texture(&mut self, texture_name: String) -> String {
        let id = Uuid::new_v4().to_string();
        println!(
            "[l1] registering texture id {} with name {}",
            id, texture_name
        );

        self.loaded_textures.insert(
            id.clone(),
            Texture::from_name(
                &self.device,
                &self.queue,
                texture_name.as_str(),
                id.as_str(),
            ),
        );

        self.texture_id = Some(id.clone());

        id
    }

    pub fn add_text(&mut self, text: &str, font_size: f32, line_height: f32) {
        let logical_width =
            self.text_renderer.physical_size.width as f32 / self.text_renderer.scale_factor;

        let mut text_buffer = glyphon::Buffer::new(
            &mut self.text_renderer.font_system,
            Metrics::relative(font_size, line_height),
        );
        text_buffer.set_size(
            &mut self.text_renderer.font_system,
            Some(logical_width - 20.0),
            None,
        );
        text_buffer.set_text(
            &mut self.text_renderer.font_system,
            text,
            &self.text_renderer.base_font,
            glyphon::Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut self.text_renderer.font_system, false);

        let id = Uuid::new_v4();

        self.text_renderer
            .buffers
            .insert(id.to_string(), text_buffer);

        println!("adding text {} with text {}", id, text);
    }

    fn display_text(&mut self, context: &mut FrameContext, _dt_seconds: f32) {
        let scale_factor = self.text_renderer.scale_factor;

        let left = 10.0 * scale_factor;
        let mut top = 10.0 * scale_factor;

        let bounds_left = left.floor() as i32;
        let bounds_right = (self.text_renderer.physical_size.width - 10) as i32;

        let text_areas: Vec<TextArea> = self
            .text_renderer
            .buffers
            .iter()
            .map(|(_, b)| {
                let a = TextArea {
                    buffer: b,
                    left,
                    top,
                    scale: scale_factor,
                    bounds: TextBounds {
                        left: bounds_left,
                        top: top.floor() as i32,
                        right: bounds_right,
                        bottom: top.floor() as i32 + self.text_renderer.physical_size.height as i32,
                    },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                    custom_glyphs: &[],
                };

                let total_lines = b
                    .layout_runs()
                    .fold(0usize, |total_lines, _| total_lines + 1);

                top += (total_lines as f32 * b.metrics().line_height + 5.0) * scale_factor;

                a
            })
            .collect();

        self.text_renderer
            .renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.text_renderer.font_system,
                &mut self.text_renderer.atlas,
                &self.text_renderer.viewport,
                text_areas,
                &mut self.text_renderer.swash_cache,
            )
            .unwrap();

        let mut pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &context.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        pass.set_pipeline(self.render_pipeline.as_ref().unwrap());

        self.text_renderer
            .renderer
            .render(
                &self.text_renderer.atlas,
                &self.text_renderer.viewport,
                &mut pass,
            )
            .unwrap();
    }

    fn display_imgui(&mut self, context: &mut FrameContext, dt_seconds: f32) {
        let Some(imgui) = &mut self.imgui_renderer else {
            return; // not ready
        };

        // update imgui's dt time
        imgui
            .context
            .io_mut()
            .update_delta_time(Duration::from_secs_f32(dt_seconds));

        // preparing frame
        imgui
            .platform
            .prepare_frame(imgui.context.io_mut(), &self.window)
            .expect("Failed to prepare frame");

        let ui = imgui.context.frame();
        {
            let window = ui.window("Hello world");
            window
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("Hello world!");
                    ui.text("This...is...imgui-rs on WGPU!");
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });

            let window = ui.window("Hello too");
            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([400.0, 200.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text(format!("Frametime: {dt_seconds:?}"));
                });

            ui.show_demo_window(&mut imgui.demo_open);
        }

        if imgui.last_cursor != ui.mouse_cursor() {
            imgui.last_cursor = ui.mouse_cursor();
            imgui.platform.prepare_render(ui, &self.window);
        }

        let mut rpass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &context.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(imgui.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        imgui
            .renderer
            .render(
                imgui.context.render(),
                &self.queue,
                &self.device,
                &mut rpass,
            )
            .expect("Rendering failed");

        drop(rpass);
    }

    fn begin_frame(&mut self) -> Option<FrameContext> {
        let now = Instant::now();
        if let Some(last_time) = self.last_frame_time {
            self.delta_time = now.duration_since(last_time);
        }
        self.last_frame_time = Some(now);

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                println!("[bf] failed to acquire next swap chain texture: {:?}", e);
                return None;
            }
        };

        // interpertatie van texture
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // enqueue texture bij gpu encoder
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        Some(FrameContext {
            frame,
            view,
            encoder,
        })
    }

    fn end_frame(&mut self, context: FrameContext) {
        self.queue.submit(std::iter::once(context.encoder.finish()));

        context.frame.present();
        self.text_renderer.atlas.trim();
    }
}
