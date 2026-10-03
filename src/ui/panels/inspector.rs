use bevy_egui::egui;

use super::{EditorUi, properties::properties_panel, rendering::rendering_panel};

use crate::{data::Atlas, ui::InspectorTab};

pub fn inspector_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
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
            properties_panel(ui, editor_ui, atlas);
        }

        InspectorTab::Rendering => {
            rendering_panel(ui, editor_ui, atlas);
        }
    }
}
