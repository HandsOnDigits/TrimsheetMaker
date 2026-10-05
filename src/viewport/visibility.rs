use bevy::prelude::*;

use crate::mesh::{ImportedMeshPart, ImportedMeshRoot, MeshSceneState};

pub fn sync_mesh_visibility(
    scene_state: Res<MeshSceneState>,
    mut root_query: Query<
        (&ImportedMeshRoot, &mut Visibility),
        (With<ImportedMeshRoot>, Without<ImportedMeshPart>),
    >,
    mut part_query: Query<
        (&ImportedMeshPart, &mut Visibility),
        (With<ImportedMeshPart>, Without<ImportedMeshRoot>),
    >,
) {
    for (root, mut visibility) in &mut root_query {
        let Some(mesh) = scene_state.get(root.id) else {
            continue;
        };

        *visibility = if mesh.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (part, mut visibility) in &mut part_query {
        let Some(mesh) = scene_state.get(part.mesh_id) else {
            continue;
        };

        let Some(part_entry) = mesh
            .parts
            .iter()
            .find(|entry| entry.index == part.part_index)
        else {
            continue;
        };

        *visibility = if mesh.visible && part_entry.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
