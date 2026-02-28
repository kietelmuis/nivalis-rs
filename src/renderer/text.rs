use std::collections::HashMap;

use glyphon::{Attrs, Cache, FontSystem, SwashCache, TextAtlas};
use wgpu::MultisampleState;
use winit::dpi::PhysicalSize;

use crate::renderer::Renderer;

pub(super) struct TextRenderer<'a> {
    pub(super) physical_size: PhysicalSize<u32>,
    pub(super) scale_factor: f32,
    pub(super) font_system: FontSystem,
    pub(super) base_font: Attrs<'a>,
    pub(super) swash_cache: SwashCache,
    pub(super) viewport: glyphon::Viewport,
    pub(super) atlas: TextAtlas,
    pub(super) renderer: glyphon::TextRenderer,
    pub(super) buffers: HashMap<String, glyphon::Buffer>,
}

const COLOR_MODE: glyphon::ColorMode = glyphon::ColorMode::Accurate;

impl<'a> Renderer<'a> {
    pub(super) fn create_text_renderer(
        &mut self,
        multisample_state: MultisampleState,
        scale_factor: f32,
        physical_size: PhysicalSize<u32>,
        swapchain_format: wgpu::TextureFormat,
    ) -> TextRenderer<'a> {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let cache = Cache::new(&self.device);
        let viewport = glyphon::Viewport::new(&self.device, &cache);

        let mut atlas = TextAtlas::with_color_mode(
            &self.device,
            &self.queue,
            &cache,
            swapchain_format,
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
