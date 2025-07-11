use std::sync::Arc;
use uuid::Uuid;
use winit::window::Window;

pub mod assets;
pub mod entity;
pub mod world;

use assets::Texture;

pub struct Renderer<'a> {
    surface: Option<wgpu::Surface<'a>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    window: Option<Arc<Window>>,
    config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<wgpu::RenderPipeline>,
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

        Renderer {
            surface: Some(surface),
            device: Some(device),
            queue: Some(queue),
            config: Some(config),
            window: Some(window),
            render_pipeline: None,
        }
    }

    pub fn handle_redraw(&mut self) {
        let context = self.begin_frame().unwrap();

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

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
