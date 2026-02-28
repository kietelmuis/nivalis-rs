use std::collections::HashMap;

use glyphon::{Attrs, Cache, FontSystem, SwashCache, TextAtlas};
use wgpu::MultisampleState;
use winit::dpi::PhysicalSize;

use crate::renderer::{Renderer, TextRenderer};

const COLOR_MODE: glyphon::ColorMode = glyphon::ColorMode::Accurate;
const SWAPCHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

impl<'a> Renderer<'a> {
    pub fn create_text_renderer(
        &mut self,
        multisample_state: MultisampleState,
        scale_factor: f32,
        physical_size: PhysicalSize<u32>,
    ) -> TextRenderer<'a> {
        // text renderer
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&self.device);
        let viewport = glyphon::Viewport::new(&self.device, &cache);
        let mut atlas = TextAtlas::with_color_mode(
            &self.device,
            &self.queue,
            &cache,
            SWAPCHAIN_FORMAT,
            COLOR_MODE,
        );
        let text_renderer =
            glyphon::TextRenderer::new(&mut atlas, &self.device, multisample_state, None);

        // create default font
        let font = Attrs::new()
            .family(glyphon::Family::SansSerif)
            .weight(glyphon::Weight::NORMAL);

        TextRenderer {
            physical_size,
            scale_factor,
            font_system,
            base_font: font,
            swash_cache,
            viewport,
            atlas,
            renderer: text_renderer,
            buffers: HashMap::new(),
        }
    }
}
