use std::{
    io,
    path::{Path, PathBuf},
};

use crate::data::TextureNaming;

use super::naming;

pub struct SingleMaterialScan {
    pub name: String,
    pub textures: Vec<PathBuf>,
    pub naming: TextureNaming,
}

pub fn scan(path: &Path) -> io::Result<SingleMaterialScan> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Material")
        .to_owned();

    let naming = naming::detect_naming(&[path.to_path_buf()])?;
    let textures = naming::collect_texture_paths(path)?;

    Ok(SingleMaterialScan {
        name,
        textures,
        naming,
    })
}
