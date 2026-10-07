use bevy::prelude::*;

use crate::{
    hotkeys::{EditorAction, EditorActionMessage},
    mesh::{ImportedMeshPart, MeshSceneState},
};

use super::MeshPreview;

pub fn remove_selected_mesh_primitive(
    mut messages: MessageReader<EditorActionMessage>,
    mut mesh_preview: ResMut<MeshPreview>,
    mut mesh_scene_state: ResMut<MeshSceneState>,
    mut commands: Commands,
    parts: Query<(Entity, &ImportedMeshPart)>,
) {
    for message in messages.read() {
        if message.action != EditorAction::RemoveSelectedMeshPrimitive {
            continue;
        }

        let Some(selection) = mesh_preview.selected_primitive else {
            continue;
        };

        let Some(mesh) = mesh_scene_state.get_mut(selection.mesh_id) else {
            mesh_preview.selected_primitive = None;
            continue;
        };

        let Some(object) = mesh.scene.objects.get_mut(selection.object_index) else {
            error!(
                "Mesh {} has no object {}",
                selection.mesh_id, selection.object_index
            );
            mesh_preview.selected_primitive = None;
            continue;
        };

        if selection.primitive_index >= object.primitives.len() {
            error!(
                "Mesh {} object {} has no primitive {}",
                selection.mesh_id, selection.object_index, selection.primitive_index
            );
            mesh_preview.selected_primitive = None;
            continue;
        }

        // Find the flat mesh part corresponding to this source primitive.
        let Some(part_index) = mesh
            .parts
            .iter()
            .find(|part| {
                part.object_index == selection.object_index
                    && part.primitive_index == selection.primitive_index
            })
            .map(|part| part.index)
        else {
            error!(
                "Could not find mesh part for mesh {}, object {}, primitive {}",
                selection.mesh_id, selection.object_index, selection.primitive_index
            );
            continue;
        };

        // Find the ECS entity belonging to that flat mesh part.
        let Some(entity) = parts.iter().find_map(|(entity, part)| {
            (part.mesh_id == selection.mesh_id && part.part_index == part_index).then_some(entity)
        }) else {
            error!(
                "Could not find ECS entity for mesh part {} of mesh {}",
                part_index, selection.mesh_id
            );
            continue;
        };

        // Remove the source primitive.
        object.primitives.remove(selection.primitive_index);

        // Remove the corresponding flat mesh part.
        mesh.parts.retain(|part| part.index != part_index);

        // Fix source primitive indices for the remaining parts belonging
        // to the same source object.
        for part in &mut mesh.parts {
            if part.object_index == selection.object_index
                && part.primitive_index > selection.primitive_index
            {
                part.primitive_index -= 1;
            }
        }

        // Remove the corresponding renderable entity.
        commands.entity(entity).despawn();

        mesh_preview.selected_primitive = None;
        mesh_preview.uv_revision = mesh_preview.uv_revision.wrapping_add(1);
    }
}

pub fn despawn_all_meshes(commands: &mut Commands, scene_state: &mut MeshSceneState) {
    for mesh in &mut scene_state.meshes {
        if let Some(root) = mesh.root.take() {
            commands.entity(root).despawn();
        }

        mesh.parts.clear();
        mesh.bounds_center = Vec3::ZERO;
        mesh.bounds_radius = 0.0;
    }
}
