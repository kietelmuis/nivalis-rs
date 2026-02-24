use std::sync::Arc;

use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{assets::manager::AssetManager, renderer::Renderer};

pub struct Engine<'a> {
    renderer: Renderer<'a>,
    assets: AssetManager,
}

impl<'a> Engine<'a> {
    pub fn new(window: Arc<Window>) -> Engine<'a> {
        let mut engine = Engine {
            renderer: Renderer::new(window.clone()),
            assets: AssetManager::new(),
        };

        // test
        engine.renderer.add_text(
            format!(
                "{} using {}",
                engine.renderer.adapter_info.name, engine.renderer.adapter_info.backend
            )
            .as_str(),
            15.0,
            1.15,
        );

        engine
    }

    pub fn handle_redraw(&mut self) {
        self.renderer.handle_redraw().unwrap()
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.renderer.handle_imgui_event(event);
    }

    pub fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        self.renderer.handle_resize(size);
    }
}
