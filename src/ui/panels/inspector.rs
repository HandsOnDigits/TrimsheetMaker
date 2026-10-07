use bevy_egui::egui;

use super::{EditorUi, properties::properties_panel, rendering::rendering_panel};

use crate::{data::Atlas, history::EditorHistory, mesh::MeshSceneState, ui::InspectorTab};

pub fn inspector_panel(
    ui: &mut egui::Ui,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    history: &mut EditorHistory,
    mesh_scene_state: &MeshSceneState,
) {
    ui.heading("Inspector");

    ui.horizontal(|ui| {
        if ui
            .selectable_label(
                editor_ui.inspector_tab == InspectorTab::Properties,
                "Properties",
            )
            .clicked()
        {
            editor_ui.inspector_tab = InspectorTab::Properties;
        }

        if ui
            .selectable_label(
                editor_ui.inspector_tab == InspectorTab::Rendering,
                "Rendering",
            )
            .clicked()
        {
            editor_ui.inspector_tab = InspectorTab::Rendering;
        }
    });

    ui.separator();

    match editor_ui.inspector_tab {
        InspectorTab::Properties => {
            properties_panel(ui, editor_ui, atlas, history, mesh_scene_state);
        }

        InspectorTab::Rendering => {
            rendering_panel(ui, editor_ui, atlas);
        }
    }
}
