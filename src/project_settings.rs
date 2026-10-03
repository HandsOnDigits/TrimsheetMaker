use serde::{Deserialize, Serialize};

use crate::data::TextureNaming;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSettings {
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

    /// Helper to resolve target group with fallback
    pub fn active_texture_group<'a>(&'a self, fallback_groups: &'a [String]) -> Option<&'a str> {
        self.preview_texture_group
            .as_deref()
            .or_else(|| fallback_groups.first().map(|s| s.as_str()))
    }
}
