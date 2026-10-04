use bevy::prelude::*;

use crate::{
    data::{Atlas, AtlasMaterial},
    mesh::{
        ImportedMeshPart, MeshSceneState, create_imported_bevy_mesh,
        unwrap::{MeshUvMode, create_unwrapped_bevy_mesh},
    },
    transform::Rotation,
};

use super::{MeshPreview, MeshUvSyncState};

fn transform_material_uv(uv: [f32; 2], rotation: Rotation) -> [f32; 2] {
    match rotation {
        Rotation::Deg0 => uv,
        Rotation::Deg90 => [uv[1], 1.0 - uv[0]],
        Rotation::Deg180 => [1.0 - uv[0], 1.0 - uv[1]],
        Rotation::Deg270 => [1.0 - uv[1], uv[0]],
    }
}

pub(crate) fn atlas_uv(uv: [f32; 2], material: &AtlasMaterial, atlas: &Atlas) -> [f32; 2] {
    let uv = transform_material_uv(uv, material.transform.rotation);

    let atlas_width = atlas.width as f32;
    let atlas_height = atlas.height as f32;

    if atlas_width <= 0.0 || atlas_height <= 0.0 {
        return [0.0, 0.0];
    }

    let x = material.transform.x as f32;
    let y = material.transform.y as f32;
    let width = material.transform.width as f32;
    let height = material.transform.height as f32;

    [
        (x + uv[0] * width) / atlas_width,
        (y + (1.0 - uv[1]) * height) / atlas_height,
    ]
}

pub(crate) fn material_uvs(
    uvs: &[[f32; 2]],
    material: Option<&AtlasMaterial>,
    atlas: &Atlas,
) -> Vec<[f32; 2]> {
    let Some(material) = material else {
        return vec![[0.0, 0.0]; uvs.len()];
    };

    uvs.iter()
        .copied()
        .map(|uv| atlas_uv(uv, material, atlas))
        .collect()
}

pub fn sync_mesh_uv_mode(
    mesh_preview: Res<MeshPreview>,
    atlas: Res<Atlas>,
    mesh_scene_state: Res<MeshSceneState>,
    mut sync_state: ResMut<MeshUvSyncState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&ImportedMeshPart, &mut Mesh3d)>,
) {
    if sync_state.mesh_revision == mesh_preview.uv_revision
        && sync_state.atlas_revision == atlas.uv_revision
        && sync_state.uv_mode == mesh_preview.uv_mode
    {
        return;
    }

    sync_state.mesh_revision = mesh_preview.uv_revision;
    sync_state.atlas_revision = atlas.uv_revision;
    sync_state.uv_mode = mesh_preview.uv_mode;

    for (part, mut mesh3d) in &mut query {
        let Some(mesh_entry) = mesh_scene_state.get(part.mesh_id) else {
            continue;
        };

        let Some(primitive_index) = part.primitive_index else {
            continue;
        };

        let Some(part_entry) = mesh_entry
            .parts
            .iter()
            .find(|entry| entry.index == part.part_index)
        else {
            continue;
        };

        let object_index = part_entry.object_index;

        let Some(object) = mesh_entry.scene.objects.get(object_index) else {
            continue;
        };

        let Some(primitive) = object.primitives.get(primitive_index) else {
            continue;
        };

        let material = atlas.materials.iter().find(|material| {
            material.has_mesh_assignment(part.mesh_id, object_index, primitive_index)
        });

        match mesh_preview.uv_mode {
            MeshUvMode::Imported => {
                let uvs = material_uvs(&primitive.uvs, material, &atlas);
                mesh3d.0 = create_imported_bevy_mesh(&mut meshes, primitive, uvs);
            }

            MeshUvMode::Unwrapped => {
                if mesh_preview.unwrapped_mesh_id != Some(part.mesh_id) {
                    continue;
                }

                let Some(unwrapped_scene) = mesh_preview.unwrapped_scene.as_ref() else {
                    continue;
                };

                let Some(unwrapped_object) = unwrapped_scene.objects.get(object_index) else {
                    continue;
                };

                let Some(unwrapped) = unwrapped_object.primitives.get(primitive_index) else {
                    continue;
                };

                let uvs = material_uvs(&unwrapped.uvs, material, &atlas);

                mesh3d.0 = create_unwrapped_bevy_mesh(&mut meshes, unwrapped, uvs);
            }
        }
    }
}
