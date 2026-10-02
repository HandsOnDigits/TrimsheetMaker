use std::{collections::BTreeMap, io, path::Path};

use crate::data::{MaterialSource, TextureNaming};

use super::{classify_textures, naming};

pub struct MaterialScan {
    pub materials: BTreeMap<String, MaterialSource>,
    pub naming: TextureNaming,
}

pub fn scan(root: &Path) -> io::Result<MaterialScan> {
    let mut folders = Vec::new();

    for entry in std::fs::read_dir(root)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            folders.push(entry.path());
        }
    }

    folders.sort();

    let detected_naming = naming::detect_naming(&folders)?;

    let mut materials = BTreeMap::new();

    for folder in &folders {
        let name = folder
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unnamed")
            .to_owned();

        let texture_paths = naming::collect_texture_paths(folder)?;

        let textures = classify_textures(texture_paths, &detected_naming);

        materials.insert(
            name.clone(),
            MaterialSource {
                name,
                material_ref: None,
                textures,
            },
        );
    }

    Ok(MaterialScan {
        materials,
        naming: detected_naming,
    })
}
