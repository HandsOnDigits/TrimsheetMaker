use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use bevy_egui::egui;

use crate::{
    data::{Atlas, TextureNaming},
    ui::EditorUi,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditorAction {
    NewAtlas,
    OpenProject,
    SaveProject,
    ExportAtlas,

    ImportMultipleMaterials,
    ImportSingleMaterial,

    Undo,
    Redo,

    RemoveSelectedMaterial,
    PackAtlas,

    ResetAtlasView,

    ToggleMaterialsPanel,
    ToggleInspectorPanel,
    ToggleStatusBar,
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
pub fn execute_action(action: EditorAction, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    match action {
        EditorAction::NewAtlas => {
            *atlas = Atlas::default();

            editor_ui.project_settings.texture_naming = TextureNaming::default();

            editor_ui.selected_material = None;
            editor_ui.atlas_zoom = 1.0;
            editor_ui.atlas_pan = egui::Vec2::ZERO;
        }

        EditorAction::OpenProject => {
            // TODO: Open project implementation.
        }

        EditorAction::SaveProject => {
            // TODO: Save project implementation.
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

        EditorAction::RemoveSelectedMaterial => {
            if let Some(index) = editor_ui.selected_material {
                atlas.remove(index);
                editor_ui.selected_material = None;
            }
        }

        EditorAction::PackAtlas => {
            // TODO: Real atlas packing implementation.
        }

        EditorAction::ResetAtlasView => {
            editor_ui.atlas_zoom = 1.0;
            editor_ui.atlas_pan = egui::Vec2::ZERO;
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
    }
}
