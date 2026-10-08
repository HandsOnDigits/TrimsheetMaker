use serde::{Deserialize, Serialize};

use crate::data::TextureNaming;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightMaterial {
    pub enabled: bool,
    pub color: [f32; 4],
    pub strength: f32,
}

impl Default for HighlightMaterial {
    fn default() -> Self {
        Self {
            enabled: true,
            color: [1.0, 0.8, 0.1, 1.0],
            strength: 0.35,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightMaterials {
    pub mesh: HighlightMaterial,
    pub atlas_region: HighlightMaterial,
    pub uv: HighlightMaterial,
}

impl Default for HighlightMaterials {
    fn default() -> Self {
        Self {
            mesh: HighlightMaterial::default(),
            atlas_region: HighlightMaterial::default(),
            uv: HighlightMaterial::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub highlights: HighlightMaterials,

    pub atlas_width: u32,
    pub atlas_height: u32,
    pub padding: u32,

    #[serde(default)]
    pub texture_naming: TextureNaming,

    #[serde(default)]
    pub preview_texture_group: Option<String>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            highlights: HighlightMaterials::default(),

            atlas_width: 512,
            atlas_height: 512,
            padding: 16,

            texture_naming: TextureNaming::default(),

            preview_texture_group: None,
        }
    }
}

impl ProjectSettings {
    pub fn apply_to_atlas(&self, atlas: &mut crate::data::Atlas) {
        atlas.width = self.atlas_width;
        atlas.height = self.atlas_height;
        atlas.padding = self.padding;
    }

    pub fn sync_from_atlas(&mut self, atlas: &crate::data::Atlas) {
        self.atlas_width = atlas.width;
        self.atlas_height = atlas.height;
        self.padding = atlas.padding;
    }
}
