use serde::{Deserialize, Serialize};

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    data::Atlas,
    mesh::{MeshLight, MeshSceneState, viewport::MeshPreview},
    project,
    project_settings::ProjectSettings,
    ui::EditorUi,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditorAction {
    NewAtlas,
    OpenProject,
    SaveProject,
    SaveProjectAs,

    ExportAtlas,

    ImportMultipleMaterials,
    ImportSingleMaterial,
    ImportMesh,

    Undo,
    Redo,

    RemoveSelectedMaterial,
    PackAtlas,

    ResetViewport,

    ToggleMaterialsPanel,
    ToggleInspectorPanel,
    ToggleStatusBar,
    ToggleMeshLightFollowCamera,
    ToggleMeshWindow,
    ToggleMeshPartsVisibility,
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
) {
    match action {
        EditorAction::NewAtlas => {
            *atlas = Atlas::default();
            editor_ui.selected_material = None;
            editor_ui.selected_mesh = None;

            editor_ui.project_settings = ProjectSettings::default();
            editor_ui.project_path = None;

            editor_ui.dragging_material = None;
            editor_ui.drag_offset = egui::Vec2::ZERO;

            editor_ui.atlas_zoom = 1.0;
            editor_ui.atlas_pan = egui::Vec2::ZERO;

            crate::mesh::viewport::despawn_all_meshes(commands, mesh_scene_state);

            editor_ui.mesh_import_path = None;
            editor_ui.mesh_import_paths.clear();

            mesh_preview.target = Vec3::ZERO;
            mesh_preview.distance = 4.0;
            mesh_preview.target_distance = 4.0;
            mesh_preview.yaw = 0.0;
            mesh_preview.pitch = 0.3;
        }

        EditorAction::OpenProject => match project::open_project_dialog() {
            Ok(Some(path)) => match project::ProjectFile::load(&path) {
                Ok((mut loaded_atlas, project_settings, project_meshes)) => {
                    loaded_atlas.load_textures(images, None);

                    *atlas = loaded_atlas;
                    editor_ui.project_settings = project_settings;
                    editor_ui.project_path = Some(path.clone());

                    editor_ui.selected_material = None;
                    editor_ui.dragging_material = None;
                    editor_ui.drag_offset = egui::Vec2::ZERO;

                    editor_ui.atlas_zoom = 1.0;
                    editor_ui.atlas_pan = egui::Vec2::ZERO;

                    editor_ui.mesh_import_path = None;
                    editor_ui.mesh_import_paths.clear();

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

        EditorAction::Undo => {
            // TODO: Undo implementation.
        }

        EditorAction::Redo => {
            // TODO: Redo implementation.
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

        EditorAction::RemoveSelectedMaterial => {
            if let Some(index) = editor_ui.selected_material {
                atlas.remove(index);
                editor_ui.selected_material = None;
            }
        }

        EditorAction::PackAtlas => {
            // TODO: Real atlas packing implementation.
        }

        EditorAction::ResetViewport => {
            editor_ui.atlas_zoom = 1.0;
            editor_ui.atlas_pan = egui::Vec2::ZERO;

            mesh_preview.distance = 4.0;
            mesh_preview.target_distance = 4.0;
            mesh_preview.target = Vec3::ZERO;
            mesh_preview.yaw = 0.0;
            mesh_preview.pitch = 0.3;
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
    }
}
