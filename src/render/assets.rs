use image::GenericImageView;

pub struct Rectangle {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_name(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_name: &str,
        label: &str,
    ) -> Self {
        let file = format!("assets/textures/{}", texture_name);
        println!("[l0] loading texture at {}", file);

        let image = image::open(file).unwrap();
        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,                          // mip mapping
            sample_count: 1,                             // multisampling
            dimension: wgpu::TextureDimension::D2,       // 2d image
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // using rgba8
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(label),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Texture {
            texture,
            view,
            sampler,
        }
    }
}
