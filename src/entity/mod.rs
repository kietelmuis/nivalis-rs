pub struct Rectangle {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

pub struct Entity {
    id: u64,
    name: String,
}
