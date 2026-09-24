use bevy_egui::egui;

use super::{EditorUi, menu_action_button};

use crate::{
    data::Atlas,
    hotkeys::{EditorAction, HotkeyManager, execute_action},
};

pub fn toolbar(
    ui: &mut egui::Ui,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    hotkeys: &HotkeyManager,
) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        ui.menu_button("File", |ui| {
            if menu_action_button(ui, hotkeys, EditorAction::NewAtlas, "New Atlas") {
                execute_action(EditorAction::NewAtlas, editor_ui, atlas);

                ui.close();
            }

            if menu_action_button(ui, hotkeys, EditorAction::OpenProject, "Open Project…") {
                execute_action(EditorAction::OpenProject, editor_ui, atlas);

                ui.close();
            }

            if menu_action_button(ui, hotkeys, EditorAction::SaveProject, "Save Project") {
                execute_action(EditorAction::SaveProject, editor_ui, atlas);

                ui.close();
            }

            ui.separator();

            if menu_action_button(
                ui,
                hotkeys,
                EditorAction::ImportMultipleMaterials,
                "Import multiple Materials…",
            ) {
                execute_action(EditorAction::ImportMultipleMaterials, editor_ui, atlas);

                ui.close();
            }

            if menu_action_button(
                ui,
                hotkeys,
                EditorAction::ImportSingleMaterial,
                "Import single Material…",
            ) {
                execute_action(EditorAction::ImportSingleMaterial, editor_ui, atlas);

                ui.close();
            }

            ui.separator();

            if menu_action_button(ui, hotkeys, EditorAction::ExportAtlas, "Export Atlas…") {
                execute_action(EditorAction::ExportAtlas, editor_ui, atlas);

                ui.close();
            }
        });

        ui.separator();

        ui.menu_button("Edit", |ui| {
            if menu_action_button(
                ui,
                hotkeys,
                EditorAction::RemoveSelectedMaterial,
                "Remove Selected",
            ) {
                execute_action(EditorAction::RemoveSelectedMaterial, editor_ui, atlas);

                ui.close();
            }

            if menu_action_button(ui, hotkeys, EditorAction::PackAtlas, "Pack Atlas") {
                execute_action(EditorAction::PackAtlas, editor_ui, atlas);

                ui.close();
            }
        });

        ui.separator();

        ui.menu_button("View", |ui| {
            ui.checkbox(&mut editor_ui.show_materials, "Materials");
            ui.checkbox(&mut editor_ui.show_inspector, "Inspector");
            ui.checkbox(&mut editor_ui.show_status, "Status Bar");

            ui.separator();

            if menu_action_button(
                ui,
                hotkeys,
                EditorAction::ResetAtlasView,
                "Reset Atlas View",
            ) {
                execute_action(EditorAction::ResetAtlasView, editor_ui, atlas);

                ui.close();
            }
        });

        ui.separator();

        ui.menu_button("Settings", |ui| {
            if ui.button("Editor").clicked() {
                editor_ui.show_editor_settings_window = true;
                ui.close();
            }

            if ui.button("Project").clicked() {
                editor_ui.show_project_settings_window = true;
                ui.close();
            }
        });

        ui.separator();

        ui.label(format!("Atlas: {} × {}", atlas.width, atlas.height));
    });
}
