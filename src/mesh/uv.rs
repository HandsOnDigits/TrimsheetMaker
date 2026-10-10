use bevy::prelude::*;

use crate::{
    data::Atlas,
    mesh::{ImportedMeshPart, MeshSceneState},
    transform::{Rotation, UvTransform},
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

        let assignment = atlas.materials.iter().find_map(|material| {
            material
                .mesh_assignments
                .iter()
                .find(|assignment| {
                    assignment.mesh_id == part.mesh_id
                        && assignment.object_index == mesh_part.object_index
                        && assignment.primitive_index == mesh_part.primitive_index
                })
                .map(|assignment| (material, assignment))
        });

        if let Some((material, assignment)) = assignment {
            info!(
                "UV assignment '{}', primitive {}: region={}x{}, rotation={:?}, uv_scale={:?}, uv_rotation={}",
                object.name,
                mesh_part.primitive_index,
                material.transform.width,
                material.transform.height,
                material.transform.rotation,
                assignment.uv_transform.scale,
                assignment.uv_transform.rotation,
            );
        }

        let uvs = match assignment {
            Some((material, assignment)) => transform_uvs(
                &primitive.uvs,
                &assignment.uv_transform,
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

pub(crate) fn transform_uvs(
    source_uvs: &[[f32; 2]],
    uv_transform: &UvTransform,
    transform: &crate::transform::Transform,
    atlas_width: u32,
    atlas_height: u32,
) -> Vec<[f32; 2]> {
    let atlas_width = atlas_width as f32;
    let atlas_height = atlas_height as f32;

    if atlas_width <= 0.0 || atlas_height <= 0.0 {
        return source_uvs.to_vec();
    }

    let (x, y, width, height) = transform.atlas_rect();

    let x = x as f32;
    let y = y as f32;
    let width = width as f32;
    let height = height as f32;

    let pivot = UvTransform::uv_center(source_uvs);

    info!(
        "Atlas UV mapping: rect=({}, {}, {}x{}), atlas={}x{}, rotation={:?}",
        x, y, width, height, atlas_width, atlas_height, transform.rotation
    );

    source_uvs
        .iter()
        .map(|&source_uv| {
            let uv = uv_transform.apply(Vec2::from_array(source_uv), pivot);

            if source_uvs.len() > 0 {
                info!(
                    "UV intermediate: source={:?}, transformed={:?}, pivot={:?}",
                    source_uv, uv, pivot
                );
            }

            let local_uv = match transform.rotation {
                Rotation::Deg0 => uv,
                Rotation::Deg90 => Vec2::new(uv.y, 1.0 - uv.x),
                Rotation::Deg180 => Vec2::new(1.0 - uv.x, 1.0 - uv.y),
                Rotation::Deg270 => Vec2::new(1.0 - uv.y, uv.x),
            };

            let atlas_uv = Vec2::new(
                (x + local_uv.x * width) / atlas_width,
                (y + (1.0 - local_uv.y) * height) / atlas_height,
            );

            atlas_uv.to_array()
        })
        .collect()
}
