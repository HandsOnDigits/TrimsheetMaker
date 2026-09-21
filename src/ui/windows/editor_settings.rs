use bevy::prelude::*;
use bevy_egui::egui;

use crate::hotkeys::{CaptureResult, EditorAction, HotkeyManager};

use super::super::EditorUi;

pub fn settings_window(
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

                ui.add(
                    egui::Slider::new(&mut editor_ui.mouse_scroll_speed, 1.0..=30.0).step_by(0.1),
                );
            });

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
                                        editor_ui.hotkey_capture = None;
                                    }

                                    CaptureResult::Cancelled => {
                                        editor_ui.hotkey_capture = None;
                                    }

                                    CaptureResult::Conflict(_) => {
                                        // Keep capturing so the user
                                        // can choose another shortcut.
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
                hotkeys.set_default_bindings();
                editor_ui.hotkey_capture = None;
                editor_ui.mouse_scroll_speed = 20.0;
            }
        });

    editor_ui.show_editor_settings_window = open;
}

fn all_editable_actions() -> &'static [EditorAction] {
    &[
        EditorAction::NewAtlas,
        EditorAction::OpenProject,
        EditorAction::SaveProject,
        EditorAction::ExportAtlas,
        EditorAction::ImportMultipleMaterials,
        EditorAction::ImportSingleMaterial,
        EditorAction::RemoveSelectedMaterial,
        EditorAction::PackAtlas,
        EditorAction::ResetAtlasView,
        EditorAction::ToggleMaterialsPanel,
        EditorAction::ToggleInspectorPanel,
        EditorAction::ToggleStatusBar,
    ]
}

fn action_display_name(action: EditorAction) -> &'static str {
    match action {
        EditorAction::NewAtlas => "New Atlas",
        EditorAction::OpenProject => "Open Project",
        EditorAction::SaveProject => "Save Project",
        EditorAction::ExportAtlas => "Export Atlas",

        EditorAction::ImportMultipleMaterials => "Import Multiple Materials",

        EditorAction::ImportSingleMaterial => "Import Single Material",

        EditorAction::RemoveSelectedMaterial => "Remove Selected Material",

        EditorAction::PackAtlas => "Pack Atlas",
        EditorAction::ResetAtlasView => "Reset Atlas View",

        EditorAction::Undo => "Undo",
        EditorAction::Redo => "Redo",

        EditorAction::ToggleMaterialsPanel => "Toggle Materials Panel",

        EditorAction::ToggleInspectorPanel => "Toggle Inspector Panel",

        EditorAction::ToggleStatusBar => "Toggle Status Bar",
    }
}
