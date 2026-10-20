use serde::{Deserialize, Serialize};

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    data::Atlas,
    export::model::export_glb,
    history::{EditorHistory, EditorSnapshot},
    linking,
    mesh::{MeshLight, MeshSceneState, spawn_mesh_scene_with_visibility},
    native_dialog::{NativeDialogRequest, NativeDialogResult, NativeDialogState},
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
    DeleteMeshPrimitive,
    DeleteMeshObject,
    DeleteMesh,
    RemoveMeshAt {
        mesh_id: u64,
    },

    RemoveMeshPrimitiveAt {
        mesh_id: u64,
        object_index: usize,
        primitive_index: usize,
    },

    RemoveMeshObjectAt {
        mesh_id: u64,
        object_index: usize,
    },

    ResetViewport,

    ToggleMaterialsPanel,
    ToggleInspectorPanel,
    ToggleStatusBar,
    ToggleMeshLightFollowCamera,
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

pub fn dispatch_action(
    action: EditorAction,
    dialog: &mut NativeDialogState,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    images: &mut Assets<Image>,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
    mesh_scene_state: &mut MeshSceneState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    history: &mut EditorHistory,
    app_exit: &mut MessageWriter<AppExit>,
) {
    if editor_ui.approved_project_action == Some(action) {
        editor_ui.approved_project_action = None;
    } else if matches!(action, EditorAction::NewAtlas | EditorAction::OpenProject)
        && has_unsaved_project_content(editor_ui, atlas, mesh_scene_state)
    {
        editor_ui.pending_project_action = Some(action);
        editor_ui.show_unsaved_project_warning = true;
        return;
    }

    if dialog.active {
        return;
    }

    match action {
        EditorAction::OpenProject => {
            dialog.start(NativeDialogRequest::OpenProject);
        }

        EditorAction::SaveProject
            if editor_ui.save_before_pending_action && editor_ui.project_path.is_some() =>
        {
            let path = editor_ui.project_path.clone().unwrap();

            match project::save_project(&path, atlas, &editor_ui.project_settings, mesh_scene_state)
            {
                Ok(()) => {
                    info!("Project saved: {}", path.display());

                    editor_ui.project_dirty = false;
                    editor_ui.save_before_pending_action = false;
                    editor_ui.show_unsaved_project_warning = false;

                    if let Some(pending_action) = editor_ui.pending_project_action.take() {
                        editor_ui.approved_project_action = Some(pending_action);

                        dispatch_action(
                            pending_action,
                            dialog,
                            editor_ui,
                            atlas,
                            images,
                            mesh_preview,
                            mesh_light,
                            mesh_scene_state,
                            commands,
                            meshes,
                            history,
                            app_exit,
                        );
                    }
                }
                Err(error) => {
                    error!("Failed to save project '{}': {error}", path.display());

                    // Keep the warning open so the user can retry or cancel.
                    editor_ui.project_dirty = true;
                    editor_ui.save_before_pending_action = false;
                    editor_ui.show_unsaved_project_warning = true;
                }
            }
        }

        EditorAction::SaveProject if editor_ui.save_before_pending_action => {
            // The warning's Save button must use the managed dialog,
            // rather than opening rfd::FileDialog directly.
            dialog.start(NativeDialogRequest::SaveProjectAs);
        }

        EditorAction::SaveProjectAs => {
            dialog.start(NativeDialogRequest::SaveProjectAs);
        }

        EditorAction::ExportModel => {
            dialog.start(NativeDialogRequest::ExportModel);
        }

        _ => execute_action(
            action,
            editor_ui,
            atlas,
            images,
            mesh_preview,
            mesh_light,
            mesh_scene_state,
            commands,
            meshes,
            history,
        ),
    }
}

pub fn open_project_from_path(
    path: std::path::PathBuf,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    images: &mut Assets<Image>,
    mesh_preview: &mut MeshPreview,
    mesh_scene_state: &mut MeshSceneState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
) {
    match project::ProjectFile::load(&path, images) {
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
            editor_ui.project_dirty = false;
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
                    editor_ui.project_path = Some(path.clone());
                }
                Err(error) => {
                    error!("Project opened, but some saved meshes could not be loaded: {error}")
                }
            }
        }
        Err(error) => {
            error!("Failed to open project '{}': {error}", path.display());
        }
    }
}

pub fn handle_native_dialog_result(
    result: NativeDialogResult,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    images: &mut Assets<Image>,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
    mesh_scene_state: &mut MeshSceneState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
) -> bool {
    match result {
        NativeDialogResult::SingleMaterialFolder(Some(path)) => {
            match crate::ui::windows::importer::single_material::scan(&path) {
                Ok(scan) => {
                    editor_ui.single_material_root = Some(path);
                    editor_ui.single_material_name = scan.name;

                    editor_ui
                        .project_settings
                        .texture_naming
                        .merge_detected(scan.naming);

                    editor_ui.single_material_textures.clear();

                    let textures = crate::ui::windows::importer::classify_textures(
                        scan.textures,
                        &editor_ui.project_settings.texture_naming,
                    );

                    for texture in textures {
                        if let Some(group_id) = texture.group_id {
                            editor_ui
                                .single_material_textures
                                .insert(group_id, Some(texture.path));
                        }
                    }
                }
                Err(error) => {
                    error!("Failed to scan material folder: {error}");

                    editor_ui.single_material_root = None;
                    editor_ui.single_material_name.clear();
                    editor_ui.single_material_textures.clear();
                }
            }
        }

        NativeDialogResult::SingleMaterialFolder(None) => {}

        NativeDialogResult::SingleMaterialTexture { group_id, path } => {
            if let Some(path) = path {
                editor_ui
                    .single_material_textures
                    .insert(group_id, Some(path));
            }
        }

        NativeDialogResult::MultipleMaterialsFolder(Some(path)) => {
            match crate::ui::windows::importer::multiple_materials::scan(&path) {
                Ok(scan) => {
                    editor_ui.multiple_materials_root = Some(path);
                    editor_ui.import_materials = scan.materials;

                    editor_ui
                        .project_settings
                        .texture_naming
                        .merge_detected(scan.naming);

                    editor_ui
                        .project_settings
                        .texture_naming
                        .reclassify_materials(&mut editor_ui.import_materials);
                }
                Err(error) => {
                    error!("Failed to scan material folder: {error}");

                    editor_ui.multiple_materials_root = None;
                    editor_ui.import_materials.clear();
                    editor_ui.project_settings.texture_naming =
                        crate::data::TextureNaming::default();
                }
            }
        }

        NativeDialogResult::MultipleMaterialsFolder(None) => {}

        NativeDialogResult::MeshFiles(Some(paths)) => {
            editor_ui.mesh_import_paths = paths;
            editor_ui.mesh_import_error = None;
        }

        NativeDialogResult::MeshFiles(None) => {}

        NativeDialogResult::OpenProject(Some(path)) => {
            open_project_from_path(
                path,
                editor_ui,
                atlas,
                images,
                mesh_preview,
                mesh_scene_state,
                commands,
                meshes,
            );
        }
        NativeDialogResult::OpenProject(None) => {
            info!("Open Project cancelled");
        }

        NativeDialogResult::SaveProjectAs(Some(path)) => {
            match project::save_project(&path, atlas, &editor_ui.project_settings, mesh_scene_state)
            {
                Ok(()) => {
                    info!("Project saved: {}", path.display());
                    editor_ui.project_path = Some(path);
                    editor_ui.project_dirty = false;
                    return true;
                }
                Err(error) => {
                    error!("Failed to save project '{}': {error}", path.display());
                    return false;
                }
            }
        }

        NativeDialogResult::SaveProjectAs(None) => {
            info!("Save Project As cancelled");
            return false;
        }

        NativeDialogResult::ExportModel(Some(path)) => {
            match export_glb(&path, mesh_scene_state, atlas) {
                Ok(()) => info!("Model exported: {}", path.display()),
                Err(error) => {
                    error!("Failed to export model '{}': {error:#}", path.display());
                }
            }
        }
        NativeDialogResult::ExportModel(None) => {
            info!("Export Model cancelled");
        }
    }

    // These parameters aren't needed by every dialog result.
    let _ = (images, mesh_light);

    false
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

            editor_ui.project_path = None;
            editor_ui.project_dirty = false;

            editor_ui.inspected_atlas_region = None;
            editor_ui.dragging_atlas_region = None;
            editor_ui.drag_offset = egui::Vec2::ZERO;

            editor_ui.project_settings = ProjectSettings::default();

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
                        editor_ui.project_dirty = false;
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
                        editor_ui.project_dirty = false;
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
                    editor_ui.project_dirty = false;
                }

                Ok(None) => {
                    info!("Save Project As cancelled");
                }

                Err(error) => {
                    error!("Failed to save project: {error}");
                }
            }
        }

        EditorAction::ExportAtlas => match crate::export::atlas::export_atlas_maps(atlas, images) {
            Ok(0) => {
                info!("Export Atlas cancelled");
            }
            Ok(exported) => {
                info!("Exported {exported} atlas texture maps");
            }
            Err(error) => {
                error!("Failed to export atlas maps: {error:#}");
            }
        },

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

            editor_ui.project_dirty = true;
        }

        EditorAction::UnassignMaterial => {
            linking::unassign_selected_primitives(
                editor_ui,
                atlas,
                mesh_scene_state,
                mesh_preview,
                history,
            );

            editor_ui.project_dirty = true;
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

                // Preserve visibility and update primitive indices after deletion.
                let saved_visibility: Vec<(usize, usize, bool)> = mesh
                    .parts
                    .iter()
                    .filter_map(|part| {
                        if part.object_index != selection.object_index {
                            return Some((part.object_index, part.primitive_index, part.visible));
                        }

                        // Do not retain visibility for the deleted primitive.
                        if part.primitive_index == selection.primitive_index {
                            return None;
                        }

                        // Shift later primitives in this object down by one.
                        let primitive_index = if part.primitive_index > selection.primitive_index {
                            part.primitive_index - 1
                        } else {
                            part.primitive_index
                        };

                        Some((part.object_index, primitive_index, part.visible))
                    })
                    .collect();

                let id = mesh.id;
                let visible = mesh.visible;
                let scene = mesh.scene.clone();

                // Remove the old runtime hierarchy.
                if let Some(root) = mesh.root.take() {
                    commands.entity(root).despawn();
                }

                // Rebuild the runtime hierarchy, preserving visibility.
                let (root, parts, bounds_center, bounds_radius) = spawn_mesh_scene_with_visibility(
                    commands,
                    meshes,
                    &scene,
                    id,
                    mesh_preview.material.clone(),
                    Some(&saved_visibility),
                );

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
            let saved_visibility: Vec<(usize, usize, bool)> = mesh
                .parts
                .iter()
                .map(|part| (part.object_index, part.primitive_index, part.visible))
                .collect();

            let (root, parts, bounds_center, bounds_radius) = spawn_mesh_scene_with_visibility(
                commands,
                meshes,
                &mesh.scene,
                mesh.id,
                mesh_preview.material.clone(),
                Some(&saved_visibility),
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

        EditorAction::DeleteMeshPrimitive => {
            if let Some(selection) = mesh_preview.selected_primitive {
                execute_action(
                    EditorAction::RemoveMeshPrimitiveAt {
                        mesh_id: selection.mesh_id,
                        object_index: selection.object_index,
                        primitive_index: selection.primitive_index,
                    },
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
            }
        }

        EditorAction::DeleteMeshObject => {
            if let Some((mesh_id, object_index)) = editor_ui.selected_mesh_object {
                execute_action(
                    EditorAction::RemoveMeshObjectAt {
                        mesh_id,
                        object_index,
                    },
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
            }
        }

        EditorAction::DeleteMesh => {
            if let Some(mesh_id) = editor_ui.selected_mesh {
                execute_action(
                    EditorAction::RemoveMeshAt { mesh_id },
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
            }
        }

        EditorAction::RemoveMeshAt { mesh_id } => {
            let Some(mesh_index) = mesh_scene_state
                .meshes
                .iter()
                .position(|mesh| mesh.id == mesh_id)
            else {
                return;
            };

            let before =
                EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

            // Remove the mesh and its runtime hierarchy.
            let mesh = mesh_scene_state.meshes.remove(mesh_index);

            if let Some(root) = mesh.root {
                commands.entity(root).despawn();
            }

            // Remove every atlas assignment belonging to this mesh.
            for material in &mut atlas.materials {
                material
                    .mesh_assignments
                    .retain(|assignment| assignment.mesh_id != mesh_id);
            }

            // Clear selection if it refers to the deleted mesh.
            if editor_ui.selected_mesh == Some(mesh_id) {
                editor_ui.selected_mesh = None;
            }

            if editor_ui
                .selected_mesh_object
                .is_some_and(|(id, _)| id == mesh_id)
            {
                editor_ui.selected_mesh_object = None;
            }

            if editor_ui
                .selected_uv_assignment
                .is_some_and(|(id, _, _)| id == mesh_id)
            {
                editor_ui.selected_uv_assignment = None;
            }

            if mesh_preview.selected_mesh_id == Some(mesh_id)
                || mesh_preview
                    .selected_primitive
                    .is_some_and(|selection| selection.mesh_id == mesh_id)
            {
                mesh_preview.clear_selection();
            }

            editor_ui.expanded_meshes.remove(&mesh_id);
            editor_ui
                .expanded_mesh_objects
                .retain(|(id, _)| *id != mesh_id);

            atlas.mark_uv_dirty();
            mesh_preview.uv_revision = mesh_preview.uv_revision.wrapping_add(1);

            history.push(before);
        }

        EditorAction::RemoveMeshPrimitiveAt {
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

        EditorAction::RemoveMeshObjectAt {
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

pub fn has_unsaved_project_content(
    editor_ui: &EditorUi,
    atlas: &Atlas,
    meshes: &MeshSceneState,
) -> bool {
    editor_ui.project_dirty
        || (editor_ui.project_path.is_none()
            && (!atlas.materials.is_empty() || !meshes.meshes.is_empty()))
}
