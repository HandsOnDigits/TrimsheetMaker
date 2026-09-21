use bevy_egui::egui;

use super::EditorUi;

use crate::{
    data::Atlas,
    hotkeys::{EditorAction, execute_action},
};

pub fn toolbar(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        ui.menu_button("File", |ui| {
            if ui.button("New Atlas").clicked() {
                execute_action(EditorAction::NewAtlas, editor_ui, atlas);

                ui.close();
            }

            if ui.button("Open Project…").clicked() {
                execute_action(EditorAction::OpenProject, editor_ui, atlas);

                ui.close();
            }

            if ui.button("Save Project").clicked() {
                execute_action(EditorAction::SaveProject, editor_ui, atlas);

                ui.close();
            }

            ui.separator();

            if ui.button("Import multiple Materials…").clicked() {
                execute_action(EditorAction::ImportMultipleMaterials, editor_ui, atlas);

                ui.close();
            }

            if ui.button("Import single Material…").clicked() {
                execute_action(EditorAction::ImportSingleMaterial, editor_ui, atlas);

                ui.close();
            }

            ui.separator();

            if ui.button("Export Atlas…").clicked() {
                execute_action(EditorAction::ExportAtlas, editor_ui, atlas);

                ui.close();
            }
        });

        ui.separator();

        if ui.button("Remove Selected").clicked() {
            execute_action(EditorAction::RemoveSelectedMaterial, editor_ui, atlas);
        }

        if ui.button("Pack Atlas").clicked() {
            execute_action(EditorAction::PackAtlas, editor_ui, atlas);
        }

        ui.separator();

        ui.menu_button("View", |ui| {
            ui.checkbox(&mut editor_ui.show_materials, "Materials");
            ui.checkbox(&mut editor_ui.show_inspector, "Inspector");
            ui.checkbox(&mut editor_ui.show_status, "Status Bar");

            ui.separator();

            if ui.button("Reset Atlas View").clicked() {
                execute_action(EditorAction::ResetAtlasView, editor_ui, atlas);

                ui.close();
            }
        });

        ui.separator();

        ui.label(format!("Atlas: {} × {}", atlas.width, atlas.height));
    });
}
