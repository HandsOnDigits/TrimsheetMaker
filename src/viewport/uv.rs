use bevy::prelude::*;

use crate::{
    data::Atlas,
    mesh::{ImportedMeshPart, MeshSceneState},
    transform::Rotation,
};

#[derive(Resource, Default)]
pub struct MeshUvSyncState {
    pub atlas_revision: u64,
    pub initialized: bool,
}

pub fn sync_mesh_uvs(
    atlas: Res<Atlas>,
    mesh_scene_state: Res<MeshSceneState>,
    mut sync_state: ResMut<MeshUvSyncState>,
    mut meshes: ResMut<Assets<Mesh>>,
    parts: Query<&ImportedMeshPart>,
) {
    let needs_sync = !sync_state.initialized || sync_state.atlas_revision != atlas.uv_revision;

    if !needs_sync {
        return;
    }

    for part in &parts {
        let Some(mesh_entry) = mesh_scene_state.get(part.mesh_id) else {
            continue;
        };

        let Some(mesh_part) = mesh_entry
            .parts
            .iter()
            .find(|entry| entry.index == part.part_index)
        else {
            continue;
        };

        let Some(object) = mesh_entry.scene.objects.get(mesh_part.object_index) else {
            continue;
        };

        let Some(primitive) = object.primitives.get(mesh_part.primitive_index) else {
            continue;
        };

        let atlas_material = atlas.materials.iter().find(|material| {
            material.has_mesh_assignment(
                part.mesh_id,
                mesh_part.object_index,
                mesh_part.primitive_index,
            )
        });

        let uvs = match atlas_material {
            Some(material) => transform_uvs(
                &primitive.uvs,
                &material.transform,
                atlas.width,
                atlas.height,
            ),
            None => primitive.uvs.clone(),
        };

        if let Some(mut mesh) = meshes.get_mut(&part.source_mesh) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        }
    }

    sync_state.atlas_revision = atlas.uv_revision;
    sync_state.initialized = true;
}

fn transform_uvs(
    source_uvs: &[[f32; 2]],
    transform: &crate::transform::Transform,
    atlas_width: u32,
    atlas_height: u32,
) -> Vec<[f32; 2]> {
    let atlas_width = atlas_width as f32;
    let atlas_height = atlas_height as f32;

    if atlas_width <= 0.0 || atlas_height <= 0.0 {
        return source_uvs.to_vec();
    }

    let x = transform.x as f32;
    let y = transform.y as f32;
    let width = transform.width as f32;
    let height = transform.height as f32;

    source_uvs
        .iter()
        .map(|&uv| {
            let uv = match transform.rotation {
                Rotation::Deg0 => uv,
                Rotation::Deg90 => [uv[1], 1.0 - uv[0]],
                Rotation::Deg180 => [1.0 - uv[0], 1.0 - uv[1]],
                Rotation::Deg270 => [1.0 - uv[1], uv[0]],
            };

            [
                (x + uv[0] * width) / atlas_width,
                (y + (1.0 - uv[1]) * height) / atlas_height,
            ]
        })
        .collect()
}
