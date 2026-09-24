use serde::{Deserialize, Serialize};

use crate::data::TextureNaming;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub padding: u32,

    pub texture_naming: TextureNaming,

    pub preview_texture_group: Option<String>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            atlas_width: 512,
            atlas_height: 512,
            padding: 16,

            texture_naming: TextureNaming::default(),

            preview_texture_group: None,
        }
    }
}
