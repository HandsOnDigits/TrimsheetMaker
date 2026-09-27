use bevy::prelude::*;
use bevy_egui::egui;

use crate::hotkeys::{CaptureResult, EditorAction, HotkeyManager};

use super::super::{EditorSettings, EditorUi};

pub fn editor_settings_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    hotkeys: &mut HotkeyManager,
    keyboard: &ButtonInput<KeyCode>,
) {
    let mut open = editor_ui.show_editor_settings_window;

    egui::Window::new("Editor Settings")
        .open(&mut open)
        .resizable(true)
        .default_width(560.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Mouse Scroll Speed");

                let response = ui.add(
                    egui::Slider::new(
                        &mut editor_ui.editor_settings.mouse_scroll_speed,
                        1.0..=100.0,
                    )
                    .step_by(0.1),
                );

                if response.changed() {
                    editor_ui.editor_settings.save();
                }
            });

            ui.separator();

            ui.heading("Mesh Import");

            ui.horizontal(|ui| {
                ui.label("Assimp Library");

                let mut path_text = editor_ui
                    .editor_settings
                    .assimp_library_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default();

                let response =
                    ui.add(egui::TextEdit::singleline(&mut path_text).desired_width(350.0));

                if response.changed() {
                    editor_ui.mesh_import_error = None;

                    editor_ui.mesh_import_path = if path_text.trim().is_empty() {
                        None
                    } else {
                        Some(path_text.into())
                    };
                }

                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Assimp Library", &["dll", "so", "dylib"])
                        .pick_file()
                    {
                        editor_ui.editor_settings.assimp_library_path = Some(path);
                        editor_ui.editor_settings.save();
                    }
                }
            });

            if let Some(path) = &editor_ui.editor_settings.assimp_library_path {
                if !path.is_file() {
                    ui.colored_label(egui::Color32::RED, "Assimp library file does not exist.");
                }
            }

            ui.separator();

            ui.heading("Hotkeys");

            ui.label("Click a shortcut to assign a new key combination.");

            ui.add_space(8.0);

            egui::Grid::new("settings_hotkeys_grid")
                .striped(true)
                .num_columns(3)
                .spacing([24.0, 8.0])
                .show(ui, |ui| {
                    ui.strong("Action");
                    ui.strong("Shortcut");
                    ui.strong("Binding");
                    ui.end_row();

                    for &action in all_editable_actions() {
                        ui.label(action_display_name(action));

                        let is_capturing = editor_ui.hotkey_capture == Some(action);

                        if is_capturing {
                            ui.label("Press a key...");

                            if keyboard.just_pressed(KeyCode::Escape) {
                                editor_ui.hotkey_capture = None;
                            } else {
                                match hotkeys.capture_hotkey(action, keyboard) {
                                    CaptureResult::Assigned(_) => {
                                        hotkeys.save_to_settings(&mut editor_ui.editor_settings);
                                        editor_ui.editor_settings.save();

                                        editor_ui.hotkey_capture = None;
                                    }

                                    CaptureResult::Cancelled => {
                                        editor_ui.hotkey_capture = None;
                                    }

                                    CaptureResult::Conflict(_) => {
                                        // Keep capture active.
                                    }

                                    CaptureResult::None => {}
                                }
                            }
                        } else {
                            ui.label(
                                hotkeys
                                    .display_name(action)
                                    .unwrap_or_else(|| "Unassigned".to_string()),
                            );
                        }

                        if is_capturing {
                            if ui.button("Cancel").clicked() {
                                editor_ui.hotkey_capture = None;
                            }
                        } else if ui.button("Change").clicked() {
                            editor_ui.hotkey_capture = Some(action);
                        }

                        ui.end_row();
                    }
                });

            ui.separator();

            if ui.button("Reset to Defaults").clicked() {
                editor_ui.editor_settings = EditorSettings::default();

                hotkeys.set_default_bindings();
                hotkeys.save_to_settings(&mut editor_ui.editor_settings);

                editor_ui.editor_settings.save();

                editor_ui.hotkey_capture = None;
            }
        });

    editor_ui.show_editor_settings_window = open;
}

fn all_editable_actions() -> &'static [EditorAction] {
    &[
        EditorAction::NewAtlas,
        EditorAction::OpenProject,
        EditorAction::SaveProject,
        EditorAction::SaveProjectAs,
        EditorAction::ExportAtlas,
        EditorAction::ImportMultipleMaterials,
        EditorAction::ImportSingleMaterial,
        EditorAction::ImportMesh,
        EditorAction::RemoveSelectedMaterial,
        EditorAction::PackAtlas,
        EditorAction::ResetViewport,
        EditorAction::ToggleMaterialsPanel,
        EditorAction::ToggleInspectorPanel,
        EditorAction::ToggleStatusBar,
        EditorAction::ToggleMeshLightFollowCamera,
        EditorAction::ToggleMeshWindow,
    ]
}

fn action_display_name(action: EditorAction) -> &'static str {
    match action {
        EditorAction::NewAtlas => "New Atlas",
        EditorAction::OpenProject => "Open Project",
        EditorAction::SaveProject => "Save Project",
        EditorAction::SaveProjectAs => "Save Project As",
        EditorAction::ExportAtlas => "Export Atlas",

        EditorAction::ImportMultipleMaterials => "Import Multiple Materials",
        EditorAction::ImportSingleMaterial => "Import Single Material",
        EditorAction::ImportMesh => "Import Mesh",

        EditorAction::RemoveSelectedMaterial => "Remove Selected Material",

        EditorAction::PackAtlas => "Pack Atlas",
        EditorAction::ResetViewport => "Reset Atlas View",

        EditorAction::Undo => "Undo",
        EditorAction::Redo => "Redo",

        EditorAction::ToggleMaterialsPanel => "Toggle Materials Panel",
        EditorAction::ToggleInspectorPanel => "Toggle Inspector Panel",
        EditorAction::ToggleStatusBar => "Toggle Status Bar",
        EditorAction::ToggleMeshLightFollowCamera => "Toggle Mesh Light Follow Mouse",
        EditorAction::ToggleMeshWindow => "Toggle Mesh Window",
    }
}
