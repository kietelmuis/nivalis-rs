use glyphon::{Attrs, Cache, FontSystem, Metrics, SwashCache, TextArea, TextAtlas, TextBounds};
use imgui::Condition;
use log::{error, info};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;
use wgpu::util::DeviceExt;
use wgpu::{AdapterInfo, BindGroupLayout, BindGroupLayoutEntry, MultisampleState};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::assets::manager::AssetPool;
use crate::assets::{NvTexture, NvTexturePool};
use crate::renderer::systems::imgui::ImguiRenderer;

const COLOR_MODE: glyphon::ColorMode = glyphon::ColorMode::Accurate;
const SWAPCHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

mod systems;

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
    surface_config: wgpu::SurfaceConfiguration,
    render_pipeline: Option<wgpu::RenderPipeline>,
    loaded_pools: Vec<NvTexturePool>,
    bind_group_layout: BindGroupLayout,

    rng: rand::rngs::ThreadRng,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    pub adapter_info: AdapterInfo,

    // renderers
    text_renderer: TextRenderer<'a>,
    imgui_renderer: Option<ImguiRenderer>,

    last_frame_time: Option<Instant>,
    delta_time: Duration,
}

struct FrameContext {
    frame: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        uv: [1.0, 0.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

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
        info!(
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
        info!("window size is {} x {}", size.width, size.height);

        // maak en zet surface config met format en mailbox present mode
        let mut surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface_config.format = SWAPCHAIN_FORMAT;
        surface_config.present_mode = wgpu::PresentMode::Mailbox;

        surface.configure(&device, &surface_config);

        // tekst renderer
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let mut atlas =
            TextAtlas::with_color_mode(&device, &queue, &cache, SWAPCHAIN_FORMAT, COLOR_MODE);
        let text_renderer =
            glyphon::TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);

        // maak font
        let font = Attrs::new()
            .family(glyphon::Family::SansSerif)
            .weight(glyphon::Weight::NORMAL);

        // zet scaling properties
        let scale_factor = window.clone().scale_factor() as f32;

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("NvTexturePool Bind Group Layout"),
            entries: &[
                // texture binding
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // sampler binding
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    VERTICES.as_ptr() as *const u8,
                    std::mem::size_of_val(VERTICES),
                )
            },
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    INDICES.as_ptr() as *const u8,
                    std::mem::size_of_val(INDICES),
                )
            },
            usage: wgpu::BufferUsages::INDEX,
        });

        let rng = rand::thread_rng();

        let mut renderer = Renderer {
            surface: surface,
            device: device,
            queue: queue,
            surface_config: surface_config,
            window: window,
            render_pipeline: None,
            loaded_pools: Vec::new(),
            bind_group_layout: bind_layout,

            vertex_buffer,
            index_buffer,
            rng,

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

            last_frame_time: None,
            delta_time: Duration::from_secs_f32(0.0),
        };

        renderer.create_pipeline();
        renderer.create_imgui();
        renderer
    }

    pub fn insert_pool(&mut self, pool: &mut AssetPool) -> usize {
        info!("adding new asset pool");

        let id = self.loaded_pools.len();

        self.loaded_pools.push(NvTexturePool {
            textures: pool
                .textures
                .iter()
                .map(|path| {
                    NvTexture::from_name(&self.device, &self.queue, &self.bind_group_layout, path)
                })
                .collect(),
            layout: self.bind_group_layout.clone(),
        });
        self.create_pipeline();

        id
    }

    pub fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        if size.height == 0 || size.width == 0 {
            return; // stop text adjustment if window size invalid
        }

        // adjust surface config based on width and height
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
        self.window.request_redraw();

        // adjust text renderer's viewport to new surface config
        self.text_renderer.viewport.update(
            &self.queue,
            glyphon::Resolution {
                width: self.surface_config.width,
                height: self.surface_config.height,
            },
        );

        // adjust the text renderer's manual scale and size
        self.text_renderer.scale_factor = self.window.scale_factor() as f32;
        self.text_renderer.physical_size = size.cast();

        let logical_width = size.width as f32 / self.text_renderer.scale_factor;

        // resize font based on new surface config
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
        self.render_image(&mut context);
        self.display_text(&mut context, dt_seconds);

        self.end_frame(context);

        Some(())
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

        info!("adding text {} with text {}", id, text);
    }

    fn render_image(&mut self, context: &mut FrameContext) {
        let pipeline = match &self.render_pipeline {
            Some(pipeline) => pipeline,
            None => {
                error!("No render pipeline");
                return;
            }
        };

        let pool = match self.loaded_pools.get(0) {
            Some(pool) => pool,
            None => {
                error!("No texture pool");
                return;
            }
        };

        let texture = match pool
            .textures
            .get(self.rng.random_range(0..pool.textures.len()))
        {
            Some(texture) => texture,
            None => {
                error!("No texture");
                return;
            }
        };

        let mut pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Image Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &context.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &texture.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..1);
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

        // draw ui
        let ui = imgui.context.frame();
        {
            let window = ui.window("nivalis debug");
            window
                .movable(true)
                .size([300.0, 100.0], Condition::FirstUseEver)
                .position([800.0, 100.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("we all love imgui");
                    ui.text(format!("Frametime: {dt_seconds:?}"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });

            ui.show_metrics_window(&mut imgui.demo_open);
        }

        // update cursor position
        if imgui.last_cursor != ui.mouse_cursor() {
            imgui.last_cursor = ui.mouse_cursor();
            imgui.platform.prepare_render(ui, &self.window);
        }

        // make a renderpass for imgui
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

        // give imgui the renderpass
        imgui
            .renderer
            .render(
                imgui.context.render(),
                &self.queue,
                &self.device,
                &mut rpass,
            )
            .expect("Rendering failed");

        // drop it after cuz its already queued
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
            Err(wgpu::SurfaceError::Outdated) | Err(wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.surface_config);
                match self.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        error!("[bf] failed after configuring: {}", e);
                        return None;
                    }
                }
            }
            Err(e) => {
                error!("[bf] failed to acquire next swap chain texture: {:?}", e);
                return None;
            }
        };

        // interpretation of texture
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // enqueue texture
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
