use bevy_egui::egui;

use super::EditorUi;
use crate::data::Atlas;

pub fn status_bar(ui: &mut egui::Ui, editor_ui: &EditorUi, atlas: &Atlas) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        ui.label(format!("Atlas {} × {}", atlas.width, atlas.height));

        ui.separator();

        ui.label(format!("{} materials", atlas.materials.len()));

        let (used_area, unused_area, utilization) = atlas.utilization_stats();

        ui.separator();

        ui.label(format!("Utilization: {:.1}%", utilization));

        ui.separator();

        ui.label(format!("Used: {} px²", used_area));

        ui.separator();

        ui.label(format!("Unused: {} px²", unused_area));

        if let Some(index) = editor_ui.inspected_atlas_region {
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
