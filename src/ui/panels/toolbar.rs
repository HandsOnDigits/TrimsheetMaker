use bevy_egui::egui;

use super::{Atlas, EditorUi};

pub fn toolbar(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        ui.menu_button("File", |ui| {
            if ui.button("New Atlas").clicked() {
                *atlas = Atlas::default();

                editor_ui.selected_material = Some(0);
                editor_ui.atlas_zoom = 1.0;
                editor_ui.atlas_pan = egui::Vec2::ZERO;

                ui.close();
            }

            if ui.button("Open Project…").clicked() {
                ui.close();
            }

            if ui.button("Save Project").clicked() {
                ui.close();
            }

            ui.separator();

            if ui.button("Export Atlas…").clicked() {
                ui.close();
            }
        });

        ui.menu_button("Edit", |ui| {
            if ui.button("Add Material").clicked() {
                let index = atlas.add_placeholder();
                editor_ui.selected_material = Some(index);

                ui.close();
            }

            if ui.button("Remove Selected").clicked() {
                if let Some(index) = editor_ui.selected_material {
                    atlas.remove(index);
                    editor_ui.selected_material = None;
                }

                ui.close();
            }
        });

        ui.separator();

        if ui
            .add_sized([130.0, 30.0], egui::Button::new("＋ Add Material"))
            .clicked()
        {
            let index = atlas.add_placeholder();
            editor_ui.selected_material = Some(index);
        }

        if ui
            .add_sized([100.0, 30.0], egui::Button::new("Pack Atlas"))
            .clicked()
        {
            // Real atlas packing goes here.
        }

        if ui
            .add_sized([90.0, 30.0], egui::Button::new("Export"))
            .clicked()
        {
            // Export implementation goes here.
        }

        ui.separator();

        ui.menu_button("View", |ui| {
            ui.checkbox(&mut editor_ui.show_materials, "Materials");

            ui.checkbox(&mut editor_ui.show_inspector, "Inspector");

            ui.checkbox(&mut editor_ui.show_status, "Status Bar");

            ui.separator();

            if ui.button("Reset Atlas View").clicked() {
                editor_ui.atlas_zoom = 1.0;
                editor_ui.atlas_pan = egui::Vec2::ZERO;

                ui.close();
            }
        });

        ui.separator();

        ui.label(format!("Atlas: {} × {}", atlas.width, atlas.height));
    });
}
