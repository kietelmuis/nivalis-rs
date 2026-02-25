use gltf::Gltf;
use log::debug;

use crate::assets::texture::NvTexture;

pub struct NvModelPool {
    pub textures: Vec<NvModel>,
    pub layout: wgpu::BindGroupLayout,
}

pub struct NvModel {
    pub texture: NvTexture,
    pub model: u8,
}

impl NvModel {
    pub fn from_name(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        model_name: &str,
    ) {
        let file = format!("assets/models/{}", model_name);
        debug!("[l0] loading texture at {}", file);

        let gltf = Gltf::open(file).unwrap();
    }
}
