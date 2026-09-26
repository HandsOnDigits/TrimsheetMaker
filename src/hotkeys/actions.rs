use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use bevy_egui::egui;

use crate::{
    data::Atlas, mesh::MeshLight, project, project_settings::ProjectSettings, ui::EditorUi,
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
    mesh_preview: &mut crate::mesh::MeshPreview,
    mesh_light: &mut MeshLight,
) {
    match action {
        EditorAction::NewAtlas => {
            *atlas = Atlas::default();
            editor_ui.project_settings = ProjectSettings::default();
            editor_ui.project_path = None;
            editor_ui.selected_material = None;
            editor_ui.atlas_zoom = 1.0;
            editor_ui.atlas_pan = egui::Vec2::ZERO;
        }

        EditorAction::OpenProject => match crate::project::open_project_dialog() {
            Ok(Some(path)) => match crate::project::ProjectFile::load(&path) {
                Ok((mut loaded_atlas, project_settings)) => {
                    loaded_atlas.load_textures(images, None);

                    *atlas = loaded_atlas;
                    editor_ui.project_settings = project_settings;
                    editor_ui.project_path = Some(path);

                    editor_ui.selected_material = None;
                    editor_ui.dragging_material = None;
                    editor_ui.drag_offset = egui::Vec2::ZERO;

                    editor_ui.atlas_zoom = 1.0;
                    editor_ui.atlas_pan = egui::Vec2::ZERO;

                    info!(
                        "Project opened: {}",
                        editor_ui.project_path.as_deref().unwrap().display()
                    );
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
                match crate::project::save_project(path, atlas, &editor_ui.project_settings) {
                    Ok(()) => {
                        info!("Project saved: {}", path.display());
                    }
                    Err(error) => {
                        error!("Failed to save project '{}': {error}", path.display());
                    }
                }
            } else {
                match crate::project::save_project_as_dialog(atlas, &editor_ui.project_settings) {
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
            match project::save_project_as_dialog(atlas, &editor_ui.project_settings) {
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
            mesh_preview.rotation = Quat::from_rotation_x(-0.25);
            mesh_preview.target = Vec3::ZERO;
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
    }
}
