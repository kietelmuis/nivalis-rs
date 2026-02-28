use std::borrow::Cow;

use log::info;
use wgpu::{RenderPipeline, ShaderSource};

use crate::renderer::Renderer;

static BASIC_SHADER: ShaderSource =
    ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders/basic.wgsl")));

#[derive(Hash, Eq, PartialEq)]
pub enum PipelineType {
    Basic2D,
    Basic3D,
}

impl<'a> Renderer<'a> {
    pub fn create_pipeline(
        &mut self,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        vertex_buffer_layouts: &[wgpu::VertexBufferLayout],
    ) -> Result<RenderPipeline, wgpu::Error> {
        info!("creating render pipeline");

        // load basic shader
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Basic Shader"),
                source: BASIC_SHADER.clone(),
            });

        // create pipeline layout
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    push_constant_ranges: &[],
                    bind_group_layouts,
                });

        // create pipeline itself
        Ok(self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: vertex_buffer_layouts,
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            }))
    }
}
