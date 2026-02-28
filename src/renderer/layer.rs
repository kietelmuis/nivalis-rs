use crate::assets::NvTexture;

pub struct Transform {
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
}

pub struct Sprite {
    transform: Transform,
    texture: NvTexture,
}

pub struct Layer<I> {
    pub instances: Vec<I>,
    pub zindex: u32,
}

impl<I> Layer<I> {
    fn draw(&self, encoder: &mut wgpu::CommandEncoder) {
        self.instances.iter().for_each(move |instance| {
            println!("Drawing instance");
        });
    }
}
