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

        egui::containers::menu::MenuButton::new("Project")
            .config(
                egui::containers::menu::MenuConfig::new()
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
            )
            .ui(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.set_width(220.0);

                    ui.label("Atlas");
                    ui.separator();

                    egui::Grid::new("project_atlas_settings")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Width");

                            if ui
                                .add_sized(
                                    [100.0, 20.0],
                                    egui::DragValue::new(
                                        &mut editor_ui.project_settings.atlas_width,
                                    )
                                    .range(1..=16384)
                                    .speed(1.0),
                                )
                                .changed()
                            {
                                atlas.width = editor_ui.project_settings.atlas_width;
                            }

                            ui.end_row();

                            ui.label("Height");

                            if ui
                                .add_sized(
                                    [100.0, 20.0],
                                    egui::DragValue::new(
                                        &mut editor_ui.project_settings.atlas_height,
                                    )
                                    .range(1..=16384)
                                    .speed(1.0),
                                )
                                .changed()
                            {
                                atlas.height = editor_ui.project_settings.atlas_height;
                            }

                            ui.end_row();

                            ui.label("Padding");

                            if ui
                                .add_sized(
                                    [100.0, 20.0],
                                    egui::DragValue::new(&mut editor_ui.project_settings.padding)
                                        .range(0..=4096)
                                        .speed(1.0),
                                )
                                .changed()
                            {
                                atlas.padding = editor_ui.project_settings.padding;
                            }

                            ui.end_row();
                        });
                });
            });

        ui.separator();

        ui.menu_button("Settings", |ui| {
            if ui.button("Editor").clicked() {
                editor_ui.show_editor_settings_window = true;
                ui.close();
            }
        });

        ui.separator();

        ui.label(format!("Atlas: {} × {}", atlas.width, atlas.height));
    });
}
