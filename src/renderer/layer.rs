use wgpu::{BindGroup, Device, RenderPipeline, util::DeviceExt, wgc::validation::BindingError};

use crate::{assets::NvTexture, renderer::FrameContext};

#[repr(C)]
#[derive(Copy, Clone)]
pub(super) struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

pub struct Transform {
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
}

impl Transform {
    pub fn calculate_vertices(&self) -> [Vertex; 4] {
        let (x, y, z) = (self.position[0], self.position[1], self.position[2]);
        let (sx, sy) = (self.scale[0], self.scale[1]);

        [
            Vertex {
                position: [x - sx / 2.0, y - sy / 2.0, z],
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [x + sx / 2.0, y - sy / 2.0, z],
                uv: [1.0, 0.0],
            },
            Vertex {
                position: [x + sx / 2.0, y + sy / 2.0, z],
                uv: [1.0, 1.0],
            },
            Vertex {
                position: [x - sx / 2.0, y + sy / 2.0, z],
                uv: [0.0, 1.0],
            },
        ]
    }
}

pub struct TexuredEntity {
    transform: Transform,
    texture: NvTexture,
}

impl Drawable for TexuredEntity {
    fn indices(&self) -> [u16; 6] {
        [0, 1, 2, 0, 2, 3]
    }

    fn vertices(&self) -> [Vertex; 4] {
        self.transform.calculate_vertices()
    }
}

pub trait Drawable {
    fn indices(&self) -> [u16; 6];
    fn vertices(&self) -> [Vertex; 4];
    fn texture(&self) -> &NvTexture;
}

pub(super) struct Layer {
    pub zindex: u32,
    bind_group: BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl Layer {
    pub fn new(
        instances: Vec<Box<dyn Drawable>>,
        device: &Device,
        bind_group: BindGroup,
        zindex: u32,
    ) -> Self {
        let indices = instances
            .iter()
            .flat_map(|i| i.indices())
            .collect::<Vec<u16>>();
        let vertices = instances
            .iter()
            .flat_map(|i| i.vertices())
            .collect::<Vec<Vertex>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Drawable Vertex Buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    vertices.as_ptr() as *const u8,
                    vertices.len() * std::mem::size_of::<Vertex>(),
                )
            },
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Drawable Index Buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    indices.as_ptr() as *const u8,
                    indices.len() * std::mem::size_of::<u16>(),
                )
            },
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            instances,
            vertex_buffer,
            index_buffer,
            bind_group,
            zindex,
            index_count: indices.len() as u32,
        }
    }

    pub fn draw(&self, context: &mut FrameContext, pipeline: &RenderPipeline) {
        let mut rpass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Drawable Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &context.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        rpass.set_pipeline(pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
