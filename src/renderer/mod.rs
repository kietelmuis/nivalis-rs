use ::imgui as imgui_lib;
use imgui_lib::Condition;

use glyphon::{Metrics, TextArea, TextBounds};
use log::{error, info};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;
use wgpu::{AdapterInfo, BindGroupLayout, BindGroupLayoutEntry, MultisampleState};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::assets::manager::AssetPool;
use crate::assets::{NvTexture, NvTexturePool};
use crate::renderer::imgui::ImguiRenderer;
use crate::renderer::pipeline::PipelineType;
use crate::renderer::text::TextRenderer;

mod imgui;
mod layer;
mod pipeline;
mod text;

const SWAPCHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    surface_config: wgpu::SurfaceConfiguration,
    loaded_pools: Vec<NvTexturePool>,
    bind_group_layouts: Vec<BindGroupLayout>,
    pipelines: HashMap<PipelineType, wgpu::RenderPipeline>,

    rng: rand::rngs::ThreadRng,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    pub adapter_info: AdapterInfo,

    // renderers
    text_renderer: Option<TextRenderer<'a>>,
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
        info!("creating renderer");

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        // choose gpu
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("failed to find graphical adapter");

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
        .expect("failed to request graphical device");

        // create surface configuration
        let size = window.clone().inner_size();
        info!("window size is {} x {}", size.width, size.height);

        let mut surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface_config.format = SWAPCHAIN_FORMAT;
        surface_config.present_mode = wgpu::PresentMode::Mailbox;

        surface.configure(&device, &surface_config);

        let mut bind_layouts = Vec::new();
        bind_layouts.push(
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            }),
        );

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

        let scale_factor = window.clone().scale_factor() as f32;

        let mut renderer = Renderer {
            surface: surface,
            device: device,
            queue: queue,
            surface_config: surface_config,
            window: window,
            loaded_pools: Vec::new(),
            bind_group_layouts: bind_layouts,
            pipelines: HashMap::new(),

            vertex_buffer,
            index_buffer,
            rng: rand::rng(),

            adapter_info: adapter.get_info(),

            imgui_renderer: None,
            text_renderer: None,

            last_frame_time: None,
            delta_time: Duration::from_secs_f32(0.0),
        };

        info!("creating pipelines");

        let basic_2d_pipeline = renderer
            .create_pipeline(
                renderer
                    .bind_group_layouts
                    .clone()
                    .iter()
                    .collect::<Vec<&BindGroupLayout>>()
                    .as_slice(),
                &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
                }],
            )
            .expect("failed to create basic 2d render pipeline");

        renderer
            .pipelines
            .insert(PipelineType::Basic2D, basic_2d_pipeline);

        renderer.text_renderer = Some(renderer.create_text_renderer(
            MultisampleState::default(),
            scale_factor,
            size,
            SWAPCHAIN_FORMAT,
        ));
        renderer.imgui_renderer = Some(
            renderer
                .create_imgui_renderer()
                .expect("failed to create imgui renderer"),
        );

        info!("renderer created");
        renderer
    }

    pub fn insert_pool(&mut self, pool: &mut AssetPool) -> usize {
        info!("adding new asset pool");

        let id = self.loaded_pools.len();
        let layout = self
            .bind_group_layouts
            .first()
            .expect("there is no bind group layout");

        self.loaded_pools.push(NvTexturePool {
            textures: pool
                .textures
                .iter()
                .map(|path| NvTexture::from_name(&self.device, &self.queue, &layout, path))
                .collect(),
            layout: layout.clone(),
        });

        id
    }

    pub fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        if size.height == 0 || size.width == 0 {
            return; // window size invalid
        }

        let text_renderer = match self.text_renderer {
            Some(ref mut text_renderer) => text_renderer,
            None => {
                error!("No text renderer to handle resize");
                return;
            }
        };

        // adjust surface config based on width and height
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
        self.window.request_redraw();

        // adjust text renderer viewport to new surface config
        text_renderer.viewport.update(
            &self.queue,
            glyphon::Resolution {
                width: self.surface_config.width,
                height: self.surface_config.height,
            },
        );

        // adjust the text renderer scale and size
        text_renderer.scale_factor = self.window.scale_factor() as f32;
        text_renderer.physical_size = size.cast();

        let logical_width = size.width as f32 / text_renderer.scale_factor;

        // resize font based on new surface config
        for (_, b) in text_renderer.buffers.iter_mut() {
            b.set_size(
                &mut text_renderer.font_system,
                Some(logical_width - 20.0),
                None,
            );
            b.shape_until_scroll(&mut text_renderer.font_system, false);
        }
    }

    pub fn handle_redraw(&mut self) -> Option<()> {
        let mut context = self.begin_frame()?;
        let dt_seconds = self.delta_time.as_secs_f32();

        self.render_image(&mut context);
        self.display_text(&mut context, dt_seconds);
        self.display_imgui(&mut context, dt_seconds);

        self.end_frame(context);

        Some(())
    }

    pub fn add_text(&mut self, text: &str, font_size: f32, line_height: f32) -> Option<usize> {
        let text_renderer = match &mut self.text_renderer {
            Some(t) => t,
            None => {
                error!("failed to create text: renderer not initialized");
                None?
            }
        };

        let logical_width = text_renderer.physical_size.width as f32 / text_renderer.scale_factor;

        let mut text_buffer = glyphon::Buffer::new(
            &mut text_renderer.font_system,
            Metrics::relative(font_size, line_height),
        );
        text_buffer.set_size(
            &mut text_renderer.font_system,
            Some(logical_width - 20.0),
            None,
        );
        text_buffer.set_text(
            &mut text_renderer.font_system,
            text,
            &text_renderer.base_font,
            glyphon::Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut text_renderer.font_system, false);

        let id = text_renderer.buffers.len();
        text_renderer.buffers.insert(id.to_string(), text_buffer);

        info!("adding text {} with id {}", text, id);
        Some(id)
    }

    fn render_image(&mut self, context: &mut FrameContext) {
        let pipeline = match self.pipelines.get(&PipelineType::Basic2D) {
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
        let text_renderer = match &mut self.text_renderer {
            Some(t) => t,
            None => {
                error!("failed to display text: renderer not initialized");
                return;
            }
        };

        let scale_factor = text_renderer.scale_factor;

        let left = 10.0 * scale_factor;
        let mut top = 10.0 * scale_factor;

        let bounds_left = left.floor() as i32;
        let bounds_right = (text_renderer.physical_size.width - 10) as i32;

        let text_areas: Vec<TextArea> = text_renderer
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
                        bottom: top.floor() as i32 + text_renderer.physical_size.height as i32,
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

        text_renderer
            .renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut text_renderer.font_system,
                &mut text_renderer.atlas,
                &text_renderer.viewport,
                text_areas,
                &mut text_renderer.swash_cache,
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

        text_renderer
            .renderer
            .render(&text_renderer.atlas, &text_renderer.viewport, &mut pass)
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
                        load: wgpu::LoadOp::Load,
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

        match &mut self.text_renderer {
            Some(t) => t.atlas.trim(),
            None => (),
        };
    }
}
