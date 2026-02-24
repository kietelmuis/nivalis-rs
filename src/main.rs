use std::sync::Arc;

use log::warn;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use crate::engine::Engine;

mod assets;
mod engine;
mod renderer;
mod util;

#[derive(Default)]
struct App<'a> {
    window: Option<Arc<Window>>,
    engine: Option<Engine<'a>>,
    attributes: WindowAttributes,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = WindowAttributes::default();
        attributes.title = "nivalis".to_string();

        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        self.window = Some(window.clone());
        self.engine = Some(Engine::new(window));
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(engine) = &mut self.engine {
            engine.handle_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => {
                warn!("stopping app");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(engine) = &mut self.engine {
                    engine.handle_resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(engine), Some(window)) = (&mut self.engine, &self.window) {
                    engine.handle_redraw();
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_module("nivalis", log::LevelFilter::Debug)
        .init();

    // begin nieuwe frame na input
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::<'static>::default();
    _ = event_loop.run_app(&mut app);
}
