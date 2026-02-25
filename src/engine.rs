use std::sync::Arc;

use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{assets::manager::AssetManager, renderer::Renderer};

pub struct Engine<'a> {
    renderer: Renderer<'a>,
    assets: AssetManager,
}

impl<'a> Engine<'a> {
    pub fn new(window: Arc<Window>) -> Engine<'a> {
        let mut renderer = Renderer::new(window.clone());
        let mut asset_manager = AssetManager::new();

        let pool = asset_manager.create_pool();
        pool.register_texture("cat.png");
        pool.register_texture("eyyab.webp");
        pool.register_texture("idiot.png");

        renderer.insert_pool(pool);

        // test
        renderer.add_text(
            format!(
                "{} using {}",
                renderer.adapter_info.name, renderer.adapter_info.backend
            )
            .as_str(),
            15.0,
            1.15,
        );

        Engine {
            renderer,
            assets: asset_manager,
        }
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
