use std::time::Instant;

use imgui::FontSource;
use imgui_wgpu::RendererConfig;
use winit::event::{Event, WindowEvent};

use crate::render::ImguiRenderer;

impl<'a> crate::render::Renderer<'a> {
    pub(in crate::render) fn create_imgui(&mut self) {
        if self.imgui_renderer.is_some() {
            return; // already created
        }

        let mut context = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::new(&mut context);
        platform.attach_window(
            context.io_mut(),
            &self.window,
            imgui_winit_support::HiDpiMode::Default,
        );
        context.set_ini_filename(None);

        let font_size = (13.0 * self.text_renderer.scale_factor) as f32;
        context.io_mut().font_global_scale = (1.0 / self.text_renderer.scale_factor) as f32;

        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        //
        // Set up dear imgui wgpu renderer
        //
        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let renderer_config = RendererConfig {
            texture_format: self.config.format,
            ..Default::default()
        };

        let renderer =
            imgui_wgpu::Renderer::new(&mut context, &self.device, &self.queue, renderer_config);
        let last_cursor = None;
        let demo_open = true;

        self.imgui_renderer = Some(ImguiRenderer {
            context,
            platform,
            renderer,
            clear_color,
            demo_open,
            last_cursor,
        })
    }

    pub fn handle_imgui_event(&mut self, event: &WindowEvent) {
        if let Some(imgui_renderer) = &mut self.imgui_renderer {
            imgui_renderer.platform.handle_event::<WindowEvent>(
                imgui_renderer.context.io_mut(),
                &self.window,
                &Event::WindowEvent {
                    window_id: self.window.id(),
                    event: event.clone(),
                },
            );
        }
    }
}
