use bevy_egui::egui;

use super::{Atlas, EditorUi};

pub fn inspector_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.heading("Inspector");

    ui.separator();

    let Some(index) = editor_ui.selected_material else {
        ui.vertical_centered(|ui| {
            ui.add_space(16.0);

            ui.label(egui::RichText::new("Nothing selected").weak().size(15.0));

            ui.add_space(4.0);

            ui.label(egui::RichText::new("Select a material from the list.").weak());
        });

        return;
    };

    let Some(material) = atlas.selected_mut(Some(index)) else {
        editor_ui.selected_material = None;
        return;
    };

    ui.label(egui::RichText::new(format!("Material #{}", material.index)).strong());

    ui.add_space(8.0);

    egui::Grid::new("material_properties")
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut material.name);
            ui.end_row();

            ui.label("Source");
            ui.label(&material.source);
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            ui.label("Position");

            ui.horizontal(|ui| {
                ui.label("X");
                ui.add(egui::DragValue::new(&mut material.x).speed(1));

                ui.label("Y");
                ui.add(egui::DragValue::new(&mut material.y).speed(1));
            });

            ui.end_row();

            ui.label("Size");

            ui.horizontal(|ui| {
                ui.label("W");
                ui.add(egui::DragValue::new(&mut material.width).speed(1));

                ui.label("H");
                ui.add(egui::DragValue::new(&mut material.height).speed(1));
            });

            ui.end_row();
        });

    ui.add_space(16.0);

    ui.separator();

    ui.label(egui::RichText::new("Texture").strong());

    ui.add_space(4.0);

    ui.label(egui::RichText::new(&material.source).weak());

    ui.add_space(8.0);

    let _ = ui.button("Replace Texture…");
}
