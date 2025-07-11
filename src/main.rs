use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

mod render;

#[derive(Default)]
struct App<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<render::Renderer<'a>>,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("Nivalis"))
                .unwrap(),
        );

        self.window = Some(window.clone());
        self.renderer = Some(render::Renderer::new(window.clone()));

        // test
        if let Some(renderer) = &mut self.renderer {
            renderer.load_texture(String::from("cat.png"));
        }

        // Request redraw if window exists
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("stopping app");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.handle_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    // begin nieuwe frame na frame klaar
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::<'static>::default();
    _ = event_loop.run_app(&mut app);
}
