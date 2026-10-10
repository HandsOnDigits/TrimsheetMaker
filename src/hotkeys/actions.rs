use serde::{Deserialize, Serialize};

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    data::Atlas,
    export::model::export_glb,
    history::{EditorHistory, EditorSnapshot},
    linking,
    mesh::{MeshLight, MeshSceneState, spawn_mesh_scene},
    project,
    project_settings::ProjectSettings,
    ui::EditorUi,
    viewport::{MeshPreview, TransformMode},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditorAction {
    NewAtlas,
    OpenProject,
    SaveProject,
    SaveProjectAs,

    ExportAtlas,
    ExportModel,

    ImportMultipleMaterials,
    ImportSingleMaterial,
    ImportMesh,

    Undo,
    Redo,

    AssignMaterial,
    UnassignMaterial,
    UnassignMeshPrimitive {
        mesh_id: u64,
        object_index: usize,
        primitive_index: usize,
    },

    RemoveAtlasRegion,
    RemoveAtlasRegionAt {
        index: usize,
    },
    RemoveSelectedMesh,

    RemoveMeshPrimitive {
        mesh_id: u64,
        object_index: usize,
        primitive_index: usize,
    },

    RemoveMeshObject {
        mesh_id: u64,
        object_index: usize,
    },

    ResetViewport,

    ToggleMaterialsPanel,
    ToggleInspectorPanel,
    ToggleStatusBar,
    ToggleMeshLightFollowCamera,
    ToggleMeshWindow,
    ToggleMeshPartsVisibility,
    ToggleUvTransformMode,

    TrimAtlasExactDimensions,
    TrimAtlasPowerOfTwo,

    RepackAtlas,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct EditorActionMessage {
    pub action: EditorAction,
}

impl EditorActionMessage {
    pub const fn new(action: EditorAction) -> Self {
        Self { action }
    }
}

/// Executes an editor action against the current editor state.
///
/// This is shared by toolbar buttons and keyboard shortcuts.
pub fn execute_action(
    action: EditorAction,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    images: &mut Assets<Image>,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
    mesh_scene_state: &mut MeshSceneState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    history: &mut EditorHistory,
) {
    match action {
        EditorAction::NewAtlas => {
            *atlas = Atlas::default();

            editor_ui.inspected_atlas_region = None;
            editor_ui.dragging_atlas_region = None;
            editor_ui.drag_offset = egui::Vec2::ZERO;

            editor_ui.project_settings = ProjectSettings::default();
            editor_ui.project_path = None;

            editor_ui.atlas_zoom = 1.0;
            editor_ui.atlas_pan = egui::Vec2::ZERO;

            crate::viewport::clear_all_meshes(commands, mesh_scene_state);

            editor_ui.mesh_import_path = None;
            editor_ui.mesh_import_paths.clear();

            mesh_preview.target = Vec3::ZERO;
            mesh_preview.distance = 4.0;
            mesh_preview.target_distance = 4.0;
            mesh_preview.yaw = 0.0;
            mesh_preview.pitch = 0.3;
            mesh_preview.selected_primitive = None;

            mesh_preview.clear_selection();

            editor_ui.clear_mesh_selection();
        }

        EditorAction::OpenProject => match project::open_project_dialog() {
            Ok(Some(path)) => match project::ProjectFile::load(&path, images) {
                Ok((mut loaded_atlas, project_settings, project_meshes)) => {
                    crate::viewport::clear_all_meshes(commands, mesh_scene_state);

                    mesh_preview.clear_selection();

                    editor_ui.clear_mesh_selection();

                    loaded_atlas.load_textures(images);

                    *atlas = loaded_atlas;

                    atlas.uv_revision = atlas.uv_revision.wrapping_add(1);
                    atlas.image_revision = atlas.image_revision.wrapping_add(1);

                    editor_ui.project_settings = project_settings;
                    editor_ui.project_path = Some(path.clone());

                    editor_ui.inspected_atlas_region = None;
                    editor_ui.dragging_atlas_region = None;
                    editor_ui.drag_offset = egui::Vec2::ZERO;

                    editor_ui.selected_mesh = None;

                    editor_ui.atlas_zoom = 1.0;
                    editor_ui.atlas_pan = egui::Vec2::ZERO;

                    editor_ui.mesh_import_path = None;
                    editor_ui.mesh_import_paths.clear();

                    mesh_preview.selected_primitive = None;

                    match project::load_project_meshes(
                        &path,
                        &project_meshes,
                        editor_ui,
                        commands,
                        meshes,
                        mesh_scene_state,
                        mesh_preview,
                    ) {
                        Ok(()) => {
                            info!("Project opened: {}", path.display());
                        }

                        Err(error) => {
                            error!(
                                "Project opened, but some saved meshes \
                                 could not be loaded: {error}"
                            );
                        }
                    }
                }

                Err(error) => {
                    error!("Failed to open project '{}': {error}", path.display());
                }
            },

            Ok(None) => {
                info!("Open Project cancelled");
            }

            Err(error) => {
                error!("Failed to open project: {error}");
            }
        },

        EditorAction::SaveProject => {
            if let Some(path) = editor_ui.project_path.as_deref() {
                match project::save_project(
                    path,
                    atlas,
                    &editor_ui.project_settings,
                    mesh_scene_state,
                ) {
                    Ok(()) => {
                        info!("Project saved: {}", path.display());
                    }

                    Err(error) => {
                        error!("Failed to save project '{}': {error}", path.display());
                    }
                }
            } else {
                match project::save_project_as_dialog(
                    atlas,
                    &editor_ui.project_settings,
                    mesh_scene_state,
                ) {
                    Ok(Some(path)) => {
                        info!("Project saved: {}", path.display());
                        editor_ui.project_path = Some(path);
                    }

                    Ok(None) => {
                        info!("Save Project cancelled");
                    }

                    Err(error) => {
                        error!("Failed to save project: {error}");
                    }
                }
            }
        }

        EditorAction::SaveProjectAs => {
            match project::save_project_as_dialog(
                atlas,
                &editor_ui.project_settings,
                mesh_scene_state,
            ) {
                Ok(Some(path)) => {
                    info!("Project saved: {}", path.display());
                    editor_ui.project_path = Some(path);
                }

                Ok(None) => {
                    info!("Save Project As cancelled");
                }

                Err(error) => {
                    error!("Failed to save project: {error}");
                }
            }
        }

        EditorAction::ExportAtlas => {
            // TODO: Atlas export implementation.
        }

        EditorAction::ExportModel => {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("glTF Binary", &["glb"])
                .set_file_name("atlas_model.glb")
                .save_file()
            else {
                info!("Export Model cancelled");
                return;
            };

            match export_glb(&path, mesh_scene_state, atlas) {
                Ok(()) => {
                    info!("Model exported: {}", path.display());
                }
                Err(error) => {
                    error!("Failed to export model '{}': {error:#}", path.display());
                }
            }
        }

        EditorAction::Undo => {
            let view = mesh_preview.get_view();

            let old_roots = mesh_scene_state
                .meshes
                .iter()
                .filter_map(|mesh| mesh.root)
                .collect::<Vec<_>>();

            if history.undo(
                atlas,
                &mut editor_ui.project_settings,
                mesh_scene_state,
                images,
            ) {
                for root in old_roots {
                    commands.entity(root).despawn();
                }

                project::restore_history_meshes(commands, meshes, mesh_scene_state, mesh_preview);

                mesh_preview.set_view(view);

                editor_ui.inspected_atlas_region = None;
                editor_ui.dragging_atlas_region = None;
                editor_ui.drag_offset = egui::Vec2::ZERO;
            }
        }

        EditorAction::Redo => {
            let view = mesh_preview.get_view();

            let old_roots = mesh_scene_state
                .meshes
                .iter()
                .filter_map(|mesh| mesh.root)
                .collect::<Vec<_>>();

            if history.redo(
                atlas,
                &mut editor_ui.project_settings,
                mesh_scene_state,
                images,
            ) {
                for root in old_roots {
                    commands.entity(root).despawn();
                }

                project::restore_history_meshes(commands, meshes, mesh_scene_state, mesh_preview);

                mesh_preview.set_view(view);

                editor_ui.inspected_atlas_region = None;
                editor_ui.dragging_atlas_region = None;
                editor_ui.drag_offset = egui::Vec2::ZERO;
            }
        }

        EditorAction::AssignMaterial => {
            linking::assign_selected_primitives(
                editor_ui,
                atlas,
                mesh_scene_state,
                mesh_preview,
                history,
            );
        }

        EditorAction::UnassignMaterial => {
            linking::unassign_selected_primitives(
                editor_ui,
                atlas,
                mesh_scene_state,
                mesh_preview,
                history,
            );
        }

        EditorAction::UnassignMeshPrimitive {
            mesh_id,
            object_index,
            primitive_index,
        } => {
            let before =
                EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

            for region in &mut atlas.materials {
                region.unassign_mesh(mesh_id, object_index, primitive_index);
            }

            atlas.uv_revision = atlas.uv_revision.wrapping_add(1);
            history.push(before);
        }

        EditorAction::ImportMultipleMaterials => {
            editor_ui.show_import_multiple_materials_window = true;
        }

        EditorAction::ImportSingleMaterial => {
            editor_ui.show_import_single_material_window = true;
        }

        EditorAction::ImportMesh => {
            editor_ui.show_import_mesh_window = true;
        }

        EditorAction::RemoveAtlasRegion => {
            if let Some(index) = editor_ui.inspected_atlas_region {
                remove_atlas_region(index, editor_ui, atlas, mesh_scene_state, history);
            }
        }

        EditorAction::RemoveAtlasRegionAt { index } => {
            remove_atlas_region(index, editor_ui, atlas, mesh_scene_state, history);
        }

        EditorAction::RemoveSelectedMesh => {
            // Primitive deletion takes priority when a primitive is selected.
            if let Some(selection) = mesh_preview.selected_primitive {
                let before =
                    EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

                let Some(mesh) = mesh_scene_state.get_mut(selection.mesh_id) else {
                    mesh_preview.selected_primitive = None;
                    return;
                };

                let Some(object) = mesh.scene.objects.get_mut(selection.object_index) else {
                    error!(
                        "Mesh {} has no object {}",
                        selection.mesh_id, selection.object_index
                    );
                    mesh_preview.selected_primitive = None;
                    return;
                };

                if selection.primitive_index >= object.primitives.len() {
                    error!(
                        "Mesh {} object {} has no primitive {}",
                        selection.mesh_id, selection.object_index, selection.primitive_index
                    );
                    mesh_preview.selected_primitive = None;
                    return;
                }

                // Remove atlas assignments belonging to this primitive.
                for material in &mut atlas.materials {
                    material.mesh_assignments.retain_mut(|assignment| {
                        if assignment.mesh_id != selection.mesh_id
                            || assignment.object_index != selection.object_index
                        {
                            return true;
                        }

                        if assignment.primitive_index == selection.primitive_index {
                            return false;
                        }

                        if assignment.primitive_index > selection.primitive_index {
                            assignment.primitive_index -= 1;
                        }

                        true
                    });
                }

                // Remove the source primitive.
                object.primitives.remove(selection.primitive_index);

                let id = mesh.id;
                let visible = mesh.visible;
                let scene = mesh.scene.clone();

                // Remove the old runtime hierarchy.
                if let Some(root) = mesh.root.take() {
                    commands.entity(root).despawn();
                }

                // Rebuild the runtime hierarchy from the updated source scene.
                let (root, parts, bounds_center, bounds_radius) =
                    spawn_mesh_scene(commands, meshes, &scene, id, mesh_preview.material.clone());

                mesh.root = Some(root);
                mesh.parts = parts;
                mesh.bounds_center = bounds_center;
                mesh.bounds_radius = bounds_radius;
                mesh.visible = visible;

                mesh_preview.selected_primitive = None;
                mesh_preview.uv_revision = mesh_preview.uv_revision.wrapping_add(1);

                atlas.mark_uv_dirty();

                // Store the state from BEFORE the deletion.
                history.push(before);

                return;
            }

            // Otherwise delete the selected object.
            let Some((mesh_id, object_index)) = editor_ui.selected_mesh_object else {
                return;
            };

            let Some(mesh_position) = mesh_scene_state
                .meshes
                .iter()
                .position(|mesh| mesh.id == mesh_id)
            else {
                editor_ui.selected_mesh_object = None;
                return;
            };

            let before =
                EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

            // Remove the object's atlas assignments and shift assignments
            // belonging to objects that come after it.
            for material in &mut atlas.materials {
                material.mesh_assignments.retain_mut(|assignment| {
                    if assignment.mesh_id != mesh_id {
                        return true;
                    }

                    if assignment.object_index == object_index {
                        return false;
                    }

                    if assignment.object_index > object_index {
                        assignment.object_index -= 1;
                    }

                    true
                });
            }

            // Remove the object from the source scene.
            let mesh = &mut mesh_scene_state.meshes[mesh_position];

            if object_index >= mesh.scene.objects.len() {
                editor_ui.selected_mesh_object = None;
                return;
            }

            mesh.scene.objects.remove(object_index);

            // The previous part coordinates are no longer reliable after removing
            // an object, so rebuild the part entries from the scene.
            //
            // Preserve visibility by matching the old entries using their old
            // coordinates before replacing the list.
            let old_parts = std::mem::take(&mut mesh.parts);

            let mut parts = Vec::new();
            let mut flat_index = 0;

            for (new_object_index, object) in mesh.scene.objects.iter().enumerate() {
                for (primitive_index, _) in object.primitives.iter().enumerate() {
                    let old_object_index = if new_object_index >= object_index {
                        new_object_index + 1
                    } else {
                        new_object_index
                    };

                    let visible = old_parts
                        .iter()
                        .find(|part| {
                            part.object_index == old_object_index
                                && part.primitive_index == primitive_index
                        })
                        .map(|part| part.visible)
                        .unwrap_or(true);

                    parts.push(crate::mesh::MeshPartEntry {
                        index: flat_index,
                        name: format!("Primitive {}", primitive_index + 1),
                        visible,
                        object_index: new_object_index,
                        primitive_index,
                    });

                    flat_index += 1;
                }
            }

            mesh.parts = parts;

            // Remove the old ECS hierarchy.
            if let Some(root) = mesh.root.take() {
                commands.entity(root).despawn();
            }

            mesh.bounds_center = Vec3::ZERO;
            mesh.bounds_radius = 0.0;

            // Recreate the ECS hierarchy from the updated source scene.
            let (root, parts, bounds_center, bounds_radius) = spawn_mesh_scene(
                commands,
                meshes,
                &mesh.scene,
                mesh.id,
                mesh_preview.material.clone(),
            );

            mesh.root = Some(root);
            mesh.parts = parts;
            mesh.bounds_center = bounds_center;
            mesh.bounds_radius = bounds_radius;

            // Remove the deleted object from the expanded-object state and shift
            // later object indices down by one.
            let mut expanded_objects = std::collections::HashSet::new();

            for &(expanded_mesh_id, expanded_object_index) in &editor_ui.expanded_mesh_objects {
                if expanded_mesh_id != mesh_id {
                    expanded_objects.insert((expanded_mesh_id, expanded_object_index));
                    continue;
                }

                if expanded_object_index == object_index {
                    continue;
                }

                let new_index = if expanded_object_index > object_index {
                    expanded_object_index - 1
                } else {
                    expanded_object_index
                };

                expanded_objects.insert((expanded_mesh_id, new_index));
            }

            editor_ui.expanded_mesh_objects = expanded_objects;

            editor_ui.selected_mesh_object = None;
            mesh_preview.selected_primitive = None;
            mesh_preview.selected_mesh_id = Some(mesh_id);
            mesh_preview.uv_revision = mesh_preview.uv_revision.wrapping_add(1);

            history.push(before);
        }

        EditorAction::RemoveMeshPrimitive {
            mesh_id,
            object_index,
            primitive_index,
        } => {
            let previous_mesh = editor_ui.selected_mesh;
            let previous_object = editor_ui.selected_mesh_object;
            let previous_primitive = mesh_preview.selected_primitive;
            let previous_preview_mesh = mesh_preview.selected_mesh_id;

            editor_ui.selected_mesh = Some(mesh_id);
            editor_ui.selected_mesh_object = None;
            mesh_preview.selected_mesh_id = Some(mesh_id);
            mesh_preview.selected_primitive = Some(crate::viewport::MeshPrimitiveSelection {
                mesh_id,
                object_index,
                primitive_index,
            });

            execute_action(
                EditorAction::RemoveSelectedMesh,
                editor_ui,
                atlas,
                images,
                mesh_preview,
                mesh_light,
                mesh_scene_state,
                commands,
                meshes,
                history,
            );

            // Restore the previous selection after deleting the target.
            editor_ui.selected_mesh = previous_mesh;
            editor_ui.selected_mesh_object = previous_object;
            mesh_preview.selected_primitive = previous_primitive;
            mesh_preview.selected_mesh_id = previous_preview_mesh;
        }

        EditorAction::RemoveMeshObject {
            mesh_id,
            object_index,
        } => {
            let previous_mesh = editor_ui.selected_mesh;
            let previous_object = editor_ui.selected_mesh_object;
            let previous_primitive = mesh_preview.selected_primitive;
            let previous_preview_mesh = mesh_preview.selected_mesh_id;

            editor_ui.selected_mesh = Some(mesh_id);
            editor_ui.selected_mesh_object = Some((mesh_id, object_index));
            mesh_preview.selected_mesh_id = Some(mesh_id);
            mesh_preview.selected_primitive = None;

            execute_action(
                EditorAction::RemoveSelectedMesh,
                editor_ui,
                atlas,
                images,
                mesh_preview,
                mesh_light,
                mesh_scene_state,
                commands,
                meshes,
                history,
            );

            // Restore the previous selection after deleting the target.
            editor_ui.selected_mesh = previous_mesh;
            editor_ui.selected_mesh_object = previous_object;
            mesh_preview.selected_primitive = previous_primitive;
            mesh_preview.selected_mesh_id = previous_preview_mesh;
        }

        EditorAction::ResetViewport => {
            editor_ui.atlas_zoom = 1.0;
            editor_ui.atlas_pan = egui::Vec2::ZERO;

            mesh_preview.distance = 4.0;
            mesh_preview.target_distance = 4.0;

            if let (Some(center), Some(normal)) = (
                mesh_preview.selected_primitive_center,
                mesh_preview.selected_primitive_normal,
            ) {
                // Orbit around the selected primitive's center.
                mesh_preview.target = center;

                // The normal points toward the visible side.
                // The camera must be positioned on that side.
                let direction = normal.normalize_or_zero();
                let horizontal_length = direction.x.hypot(direction.z);

                if horizontal_length > 1e-5 {
                    mesh_preview.yaw = direction.x.atan2(direction.z);
                }

                // The orbit offset's Y component is -sin(pitch).
                // Clamp to the same limits used by mouse orbit.
                mesh_preview.pitch = (-direction.y).atan2(horizontal_length).clamp(-1.5, 1.5);
            } else {
                mesh_preview.target = Vec3::ZERO;
                mesh_preview.yaw = 0.0;
                mesh_preview.pitch = 0.3;
            }
        }

        EditorAction::ToggleMaterialsPanel => {
            editor_ui.show_materials = !editor_ui.show_materials;
        }

        EditorAction::ToggleInspectorPanel => {
            editor_ui.show_inspector = !editor_ui.show_inspector;
        }

        EditorAction::ToggleStatusBar => {
            editor_ui.show_status = !editor_ui.show_status;
        }

        EditorAction::ToggleMeshLightFollowCamera => {
            mesh_light.follow_camera = !mesh_light.follow_camera;
        }

        EditorAction::ToggleMeshWindow => {
            editor_ui.show_meshes_window = !editor_ui.show_meshes_window;
        }

        EditorAction::ToggleMeshPartsVisibility => {
            if let Some(mesh_id) = editor_ui.selected_mesh {
                if let Some(mesh) = mesh_scene_state.get_mut(mesh_id) {
                    let all_visible = mesh.parts.iter().all(|part| part.visible);

                    for part in &mut mesh.parts {
                        part.visible = !all_visible;
                    }
                }
            }
        }

        EditorAction::ToggleUvTransformMode => {
            mesh_preview.transform_mode = match mesh_preview.transform_mode {
                TransformMode::None => TransformMode::Uv,
                TransformMode::Uv => TransformMode::None,
            };

            mesh_preview.uv_history_active = false;
        }

        EditorAction::TrimAtlasExactDimensions | EditorAction::TrimAtlasPowerOfTwo => {
            let power_of_two = action == EditorAction::TrimAtlasPowerOfTwo;

            let before =
                EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

            if atlas.trim_to_fit(power_of_two, editor_ui.trim_to_edge) {
                editor_ui.project_settings.atlas_width = atlas.width;
                editor_ui.project_settings.atlas_height = atlas.height;
                history.push(before);
            }
        }

        EditorAction::RepackAtlas => {
            let before =
                EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

            if atlas.repack() {
                editor_ui.project_settings.atlas_width = atlas.width;
                editor_ui.project_settings.atlas_height = atlas.height;

                history.push(before);
            }
        }
    }
}

fn remove_atlas_region(
    index: usize,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    mesh_scene_state: &mut MeshSceneState,
    history: &mut EditorHistory,
) {
    if index >= atlas.materials.len() {
        return;
    }

    history.push(EditorSnapshot::capture(
        atlas,
        &editor_ui.project_settings,
        mesh_scene_state,
    ));

    atlas.remove(index);

    if editor_ui.inspected_atlas_region == Some(index) {
        editor_ui.inspected_atlas_region = None;
    } else if let Some(inspected_index) = editor_ui.inspected_atlas_region
        && inspected_index > index
    {
        editor_ui.inspected_atlas_region = Some(inspected_index - 1);
    }

    if editor_ui.dragging_atlas_region == Some(index) {
        editor_ui.dragging_atlas_region = None;
        editor_ui.drag_offset = egui::Vec2::ZERO;
    } else if let Some(dragging_index) = editor_ui.dragging_atlas_region
        && dragging_index > index
    {
        editor_ui.dragging_atlas_region = Some(dragging_index - 1);
    }
}
