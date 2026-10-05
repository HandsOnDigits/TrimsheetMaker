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
    mut parts: ParamSet<(
        Query<(Entity, &ImportedMeshPart)>,
        Query<&mut ImportedMeshPart>,
    )>,
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
            continue;
        };

        if selection.primitive_index >= object.primitives.len() {
            error!(
                "Mesh {} object {} has no primitive {}",
                selection.mesh_id, selection.object_index, selection.primitive_index
            );
            continue;
        }

        // Find the flat mesh-part corresponding to this primitive.
        let part_index = mesh
            .parts
            .iter()
            .find(|part| {
                part.object_index == selection.object_index
                    && part.primitive_index == selection.primitive_index
            })
            .map(|part| part.index);

        let Some(part_index) = part_index else {
            error!(
                "Could not find mesh part for mesh {}, object {}, primitive {}",
                selection.mesh_id, selection.object_index, selection.primitive_index
            );
            continue;
        };

        // Find the ECS entity for that mesh part.
        let primitive_entity = parts.p0().iter().find_map(|(entity, part)| {
            (part.mesh_id == selection.mesh_id && part.part_index == part_index).then_some(entity)
        });

        let Some(primitive_entity) = primitive_entity else {
            error!(
                "Could not find ECS entity for mesh part {} of mesh {}",
                part_index, selection.mesh_id
            );
            continue;
        };

        // Remove the primitive from the imported scene.
        object.primitives.remove(selection.primitive_index);

        // Remove the corresponding mesh-part entry.
        mesh.parts.retain(|part| part.index != part_index);

        // The remaining primitives in the same source object shift down
        // by one primitive index.
        for part in &mut mesh.parts {
            if part.object_index == selection.object_index
                && part.primitive_index > selection.primitive_index
            {
                part.primitive_index -= 1;
            }
        }

        // Remove the renderable ECS entity.
        commands.entity(primitive_entity).despawn();

        // Update the primitive indices stored on the remaining ECS parts.
        for mut part in parts.p1().iter_mut() {
            if part.mesh_id == selection.mesh_id
                && part.part_index != part_index
                && part.primitive_index > selection.primitive_index
            {
                // Only update parts belonging to the same source object.
                let Some(mesh_entry) = mesh_scene_state.get(selection.mesh_id) else {
                    continue;
                };

                let Some(mesh_part) = mesh_entry
                    .parts
                    .iter()
                    .find(|entry| entry.index == part.part_index)
                else {
                    continue;
                };

                if mesh_part.object_index == selection.object_index {
                    part.primitive_index -= 1;
                }
            }
        }

        mesh_preview.selected_primitive = None;
        mesh_preview.uv_revision = mesh_preview.uv_revision.wrapping_add(1);
    }
}

pub fn despawn_all_meshes(commands: &mut Commands, scene_state: &mut MeshSceneState) {
    for mesh in scene_state.meshes.drain(..) {
        if let Some(root) = mesh.root {
            commands.entity(root).despawn();
        }
    }

    scene_state.next_id = 0;
}
