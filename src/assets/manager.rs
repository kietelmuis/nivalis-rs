use core::fmt;

use crate::assets::AssetType;

pub struct Asset {
    pub file_name: String,
    pub bundle_id: usize,
}

pub struct AssetBundle {
    pub assets: Vec<Asset>,
    pub bundle_type: AssetType,
    pub bundle_id: usize,
}

impl AssetBundle {
    pub fn new(bundle_type: AssetType, bundle_id: usize) -> Self {
        AssetBundle {
            assets: Vec::new(),
            bundle_type,
            bundle_id,
        }
    }

    pub fn register(&mut self, asset_name: &str) -> usize {
        let asset_id = self.assets.len();

        self.assets.push(Asset {
            file_name: asset_name.to_string(),
            bundle_id: self.bundle_id,
        });
        asset_id
    }

    pub fn unregister(&mut self, id: usize) {
        self.assets.remove(id);
    }
}

pub struct AssetManager {
    asset_bundles: Vec<AssetBundle>,
}

impl AssetManager {
    pub fn new() -> AssetManager {
        AssetManager {
            asset_bundles: Vec::new(),
        }
    }

    pub fn create_bundle(&mut self, pool_type: AssetType) -> &mut AssetBundle {
        let id = self.asset_bundles.len();

        self.asset_bundles.push(AssetBundle::new(pool_type, id));
        self.asset_bundles.get_mut(id).unwrap()
    }
}
