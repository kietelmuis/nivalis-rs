use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowAttributes, WindowId},
};

mod render;
mod util;

#[derive(Default)]
struct App<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<render::Renderer<'a>>,
    attributes: WindowAttributes,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // todo: should be selectable
        let monitor = event_loop.primary_monitor();

        let mut attrs = WindowAttributes::default();
        attrs.title = "nivalis".to_string();
        attrs.fullscreen = Some(Fullscreen::Borderless(monitor));
        self.attributes = attrs;

        let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());

        self.window = Some(window.clone());
        self.renderer = Some(render::Renderer::new(window.clone()));

        // test
        if let Some(renderer) = &mut self.renderer {
            renderer.load_texture(String::from("cat.png"));
            renderer.add_text(
                format!(
                    "{} using {}",
                    renderer.adapter_info.name, renderer.adapter_info.backend
                )
                .as_str(),
                15.0,
                1.15,
            );
        }

        // Request redraw if window exists
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(renderer) = &mut self.renderer {
            renderer.handle_imgui_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("stopping app");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.handle_resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                    renderer.handle_redraw();
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    // begin nieuwe frame na input
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::<'static>::default();
    _ = event_loop.run_app(&mut app);
}
