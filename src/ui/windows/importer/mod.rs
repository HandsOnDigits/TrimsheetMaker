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

            let group_id = naming.group_for_filename(filename)?;

            Some(MaterialTexture {
                id: uuid::Uuid::new_v4().to_string(),
                path,
                group_id: Some(group_id),
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
        .map(|(group_id, path)| MaterialTexture {
            id: uuid::Uuid::new_v4().to_string(),
            path,
            group_id: Some(group_id),
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
