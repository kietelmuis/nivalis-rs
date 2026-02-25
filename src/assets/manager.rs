pub struct AssetPool {
    pub textures: Vec<String>,
}

impl AssetPool {
    pub fn new() -> Self {
        AssetPool {
            textures: Vec::new(),
        }
    }

    pub fn register_texture(&mut self, path: &str) -> usize {
        let full_path = format!("textures/{}", path);
        let id = self.textures.len();

        self.textures.push(full_path);
        id
    }

    pub fn unregister_texture(&mut self, id: usize) {
        self.textures.remove(id);
    }
}

pub struct AssetManager {
    asset_pools: Vec<AssetPool>,
}

impl AssetManager {
    pub fn new() -> AssetManager {
        AssetManager {
            asset_pools: Vec::new(),
        }
    }

    pub fn create_pool(&mut self) -> &mut AssetPool {
        let id = self.asset_pools.len();

        self.asset_pools.push(AssetPool::new());
        self.asset_pools.get_mut(id).unwrap()
    }
}
