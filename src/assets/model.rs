use gltf::Gltf;
use log::debug;

use crate::assets::manager::Asset;

pub struct NvModelPool {
    pub textures: Vec<NvModel>,
    pub layout: wgpu::BindGroupLayout,
}

pub struct NvModel {
    pub buffers: Vec<Vec<u8>>,
}

impl NvModel {
    pub fn from_gltf(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        model_asset: &Asset,
    ) -> Self {
        let file = format!("assets/models/{}", model_asset.file_name);
        debug!("[l0] loading texture at {}", file);

        let gltf = Gltf::open(file).expect("failed to open gltf file");

        NvModel {
            buffers: gltf
                .buffers()
                .map(|b| gltf::buffer::Data::from_source(b.source(), None).unwrap().0)
                .collect::<Vec<Vec<u8>>>(),
        }
    }
}
