use std::path::PathBuf;

pub mod materials;

pub struct MaterialSource {
    pub name: String,
    pub textures: Vec<MaterialTextureSource>,
}

pub struct MaterialTextureSource {
    pub group: String,
    pub path: PathBuf,
}
