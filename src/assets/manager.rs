use std::collections::HashMap;

pub struct TextureHandle(pub u32);

pub struct AssetManager {
    texture_paths: HashMap<u32, String>,
    next_id: u32,
}

impl AssetManager {
    pub fn register_texture(&mut self, path: &str) -> TextureHandle {
        let id = self.next_id;
        self.texture_paths.insert(id, path.to_string());
        self.next_id += 1;
        TextureHandle(id)
    }

    pub fn new() -> AssetManager {
        AssetManager {
            texture_paths: HashMap::new(),
            next_id: 0,
        }
    }
}
