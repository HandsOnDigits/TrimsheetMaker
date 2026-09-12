use bevy_egui::egui;

use super::EditorUi;

use crate::data::Atlas;

pub fn status_bar(ui: &mut egui::Ui, editor_ui: &EditorUi, atlas: &Atlas) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        ui.label(format!("Atlas {} × {}", atlas.width, atlas.height));

        ui.separator();

        ui.label(format!("{} materials", atlas.materials.len()));

        if let Some(index) = editor_ui.selected_material {
            ui.separator();

            ui.label(format!("Selected: #{}", index));
        }

        ui.separator();

        ui.label(egui::RichText::new("Ready").weak());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("Zoom {:.0}%", editor_ui.atlas_zoom * 100.0));
        });
    });
}
