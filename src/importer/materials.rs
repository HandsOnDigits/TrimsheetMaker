use std::path::{Path, PathBuf};

use crate::data::TextureNaming;

pub struct MaterialSource {
    pub name: String,
    pub folder: PathBuf,
    pub textures: Vec<MaterialTextureSource>,
}

pub struct MaterialTextureSource {
    pub group: String,
    pub path: PathBuf,
}

pub fn scan(root: &Path, naming: &TextureNaming) -> std::io::Result<Vec<MaterialSource>> {
    let mut materials = Vec::new();

    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        materials.push(scan_material(&path, naming)?);
    }

    materials.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(materials)
}

fn scan_material(folder: &Path, naming: &TextureNaming) -> std::io::Result<MaterialSource> {
    let name = folder
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Material")
        .to_owned();

    let mut textures = Vec::new();

    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };

        let Some(group) = naming.group_for_filename(stem) else {
            continue;
        };

        textures.push(MaterialTextureSource { group, path });
    }

    Ok(MaterialSource {
        name,
        folder: folder.to_owned(),
        textures,
    })
}
