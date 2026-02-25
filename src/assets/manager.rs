use crate::assets::AssetType;

pub struct AssetPool {
    pub assets: Vec<String>,
    pub pool_type: AssetType,
}

impl AssetPool {
    pub fn new(pool_type: AssetType) -> Self {
        AssetPool {
            assets: Vec::new(),
            pool_type: pool_type,
        }
    }

    pub fn register(&mut self, asset_name: &str) -> usize {
        let id = self.assets.len();

        self.assets.push(asset_name.to_string());
        id
    }

    pub fn unregister(&mut self, id: usize) {
        self.assets.remove(id);
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

    pub fn create_pool(&mut self, pool_type: AssetType) -> &mut AssetPool {
        let id = self.asset_pools.len();

        self.asset_pools.push(AssetPool::new(pool_type));
        self.asset_pools.get_mut(id).unwrap()
    }
}
