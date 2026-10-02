pub mod multiple_materials;
mod naming;
pub mod single_material;

use std::path::PathBuf;

use crate::{
    data::{MaterialSource, MaterialTexture, TextureNaming},
    mesh::import::MeshMaterialRef,
};

pub fn classify_textures(paths: Vec<PathBuf>, naming: &TextureNaming) -> Vec<MaterialTexture> {
    paths
        .into_iter()
        .filter_map(|path| {
            let filename = path.file_name()?.to_str()?;
            let id = naming.group_for_filename(filename)?;

            Some(MaterialTexture {
                id,
                path,
                image: None,
                egui_texture: None,
            })
        })
        .collect()
}

pub fn material_source_from_paths(
    name: String,
    material_ref: Option<MeshMaterialRef>,
    textures: impl IntoIterator<Item = (String, PathBuf)>,
) -> MaterialSource {
    let textures = textures
        .into_iter()
        .map(|(id, path)| MaterialTexture {
            id,
            path,
            image: None,
            egui_texture: None,
        })
        .collect();

    MaterialSource {
        name,
        material_ref,
        textures,
    }
}
