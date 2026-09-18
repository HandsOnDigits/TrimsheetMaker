use bevy_egui::egui;

use super::EditorUi;

use crate::data::Atlas;

pub fn toolbar(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        ui.menu_button("File", |ui| {
            if ui.button("New Atlas").clicked() {
                *atlas = Atlas::default();

                editor_ui.selected_material = None;
                editor_ui.atlas_zoom = 1.0;
                editor_ui.atlas_pan = egui::Vec2::ZERO;

                ui.close();
            }

            if ui.button("Open Project…").clicked() {
                todo!(/*Open project implementation goes here.*/);
            }

            if ui.button("Save Project").clicked() {
                todo!(/*Save implementation goes here.*/);
            }

            ui.separator();

            if ui.button("Import multiple Materials…").clicked() {
                editor_ui.show_import_multiple_materials_window = true;
                ui.close();
            }

            if ui.button("Import single Material…").clicked() {
                editor_ui.show_import_single_material_window = true;
                ui.close();
            }

            ui.separator();

            if ui.button("Export Atlas…").clicked() {
                todo!(/*Atlas export implementation goes here.*/);
            }
        });

        ui.separator();

        if ui.button("Remove Selected").clicked() {
            if let Some(index) = editor_ui.selected_material {
                atlas.remove(index);
                editor_ui.selected_material = None;
            }
        }

        if ui.button("Pack Atlas").clicked() {
            todo!(/*Real atlas packing goes here.*/);
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
