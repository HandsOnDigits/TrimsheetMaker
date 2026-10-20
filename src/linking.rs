use bevy::prelude::*;

use crate::{
    data::Atlas,
    history::{EditorHistory, EditorSnapshot},
    mesh::MeshSceneState,
    ui::EditorUi,
    viewport::{MeshPreview, MeshPrimitiveSelection},
};

fn selected_primitives(
    editor_ui: &EditorUi,
    mesh_scene_state: &MeshSceneState,
    mesh_preview: &MeshPreview,
) -> Vec<MeshPrimitiveSelection> {
    if let Some(selection) = mesh_preview.selected_primitive {
        return vec![selection];
    }

    let Some((mesh_id, object_index)) = editor_ui.selected_mesh_object else {
        return Vec::new();
    };

    mesh_scene_state
        .meshes
        .iter()
        .find(|mesh| mesh.id == mesh_id)
        .map(|mesh| {
            mesh.parts
                .iter()
                .filter(|part| part.object_index == object_index)
                .map(|part| MeshPrimitiveSelection {
                    mesh_id,
                    object_index,
                    primitive_index: part.primitive_index,
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn assign_selected_primitives(
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    mesh_scene_state: &MeshSceneState,
    mesh_preview: &MeshPreview,
    history: &mut EditorHistory,
) {
    let Some(region_index) = editor_ui.inspected_atlas_region else {
        return;
    };

    let Some(region_exists) = atlas
        .materials
        .iter()
        .any(|region| region.index == region_index)
        .then_some(())
    else {
        return;
    };
    let _ = region_exists;

    let selected = selected_primitives(editor_ui, mesh_scene_state, mesh_preview);

    if selected.is_empty() {
        return;
    }

    let already_assigned = selected.iter().all(|selection| {
        atlas.materials.iter().any(|region| {
            region.index == region_index
                && region.has_mesh_assignment(
                    selection.mesh_id,
                    selection.object_index,
                    selection.primitive_index,
                )
        })
    });

    if already_assigned {
        return;
    }

    let before = EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

    for selection in &selected {
        for region in &mut atlas.materials {
            region.unassign_mesh(
                selection.mesh_id,
                selection.object_index,
                selection.primitive_index,
            );
        }
    }

    if let Some(region) = atlas
        .materials
        .iter_mut()
        .find(|region| region.index == region_index)
    {
        for selection in &selected {
            region.assign_mesh(
                selection.mesh_id,
                selection.object_index,
                selection.primitive_index,
            );
        }
    }

    atlas.uv_revision = atlas.uv_revision.wrapping_add(1);
    history.push(before);
    editor_ui.project_dirty = true;
}

pub fn unassign_selected_primitives(
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    mesh_scene_state: &MeshSceneState,
    mesh_preview: &MeshPreview,
    history: &mut EditorHistory,
) {
    let selected = selected_primitives(editor_ui, mesh_scene_state, mesh_preview);

    if selected.is_empty() {
        return;
    }

    let has_assignments = selected.iter().any(|selection| {
        atlas.materials.iter().any(|region| {
            region.has_mesh_assignment(
                selection.mesh_id,
                selection.object_index,
                selection.primitive_index,
            )
        })
    });

    if !has_assignments {
        return;
    }

    let before = EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

    for selection in &selected {
        for region in &mut atlas.materials {
            region.unassign_mesh(
                selection.mesh_id,
                selection.object_index,
                selection.primitive_index,
            );
        }
    }

    atlas.uv_revision = atlas.uv_revision.wrapping_add(1);
    history.push(before);
    editor_ui.project_dirty = true;
}
