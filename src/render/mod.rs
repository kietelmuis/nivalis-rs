use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;
use winit::window::Window;

pub mod assets;
pub mod entity;
pub mod world;

use crate::util;
use crate::util::ext::ColorExtensions;
use assets::Texture;

pub struct Renderer<'a> {
    surface: Option<wgpu::Surface<'a>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    window: Option<Arc<Window>>,
    config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<wgpu::RenderPipeline>,

    rng: rand::rngs::ThreadRng,

    current_color: wgpu::Color,
    target_color: wgpu::Color,
    transition_speed: f32,
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

        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        config.present_mode = wgpu::PresentMode::Mailbox;

        surface.configure(&device, &config);

        let mut rng = rand::rng();

        let target = wgpu::Color {
            r: rng.random_range(0.0..1.0),
            g: rng.random_range(0.0..1.0),
            b: rng.random_range(0.0..1.0),
            a: 1.0,
        };

        let current = wgpu::Color {
            r: rng.random_range(0.0..1.0),
            g: rng.random_range(0.0..1.0),
            b: rng.random_range(0.0..1.0),
            a: 1.0,
        };

        println!("{:?}", current);
        println!("{:?}", target);

        Renderer {
            surface: Some(surface),
            device: Some(device),
            queue: Some(queue),
            config: Some(config),
            window: Some(window),
            render_pipeline: None,

            rng: rng,
            current_color: current,
            target_color: target,
            transition_speed: 0.001,
        }
    }

    pub fn handle_redraw(&mut self) {
        let mut context = self.begin_frame().unwrap();

        self.display_rand(&mut context);
        // self.render_scene(&context);
        // self.render_ui(&context);

        self.end_frame(context);
    }

    pub fn load_texture(&mut self, texture_name: String) -> Texture {
        let id = Uuid::new_v4().to_string();
        println!("[l1] creating texture {}", id);

        Texture::from_name(
            &self.device.as_ref().unwrap(),
            &self.queue.as_ref().unwrap(),
            texture_name.as_str(),
            id.as_str(),
        )
    }

    fn display_rand(&mut self, context: &mut FrameContext) {
        println!("{:?}", self.current_color);
        println!("{:?}", self.target_color);

        self.current_color = self
            .current_color
            .lerp(&self.target_color, self.transition_speed);

        context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &context.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.current_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

        if self.current_color.is_near(&self.target_color, 0.001) {
            println!("reached target");
            self.target_color = wgpu::Color::random(&mut self.rng);
        }
    }

    fn begin_frame(&mut self) -> Option<FrameContext> {
        println!("[l1] begin frame");
        let (surface, device) = match (&mut self.surface, &self.device) {
            (Some(s), Some(d)) => (s, d),
            _ => return None,
        };

        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Failed to acquire next swap chain texture: {:?}", e);
                return None;
            }
        };

        // interpertatie van texture
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // enqueue texture bij gpu encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        Some(FrameContext {
            frame,
            view,
            encoder,
        })
    }

    fn end_frame(&mut self, context: FrameContext) {
        println!("[l1] end frame");
        self.queue
            .as_ref()
            .unwrap()
            .submit(std::iter::once(context.encoder.finish()));

        context.frame.present();

        // Note: frame, view, and encoder are consumed when this function returns
        // as they are part of the FrameContext struct which is moved into this function
    }
}
