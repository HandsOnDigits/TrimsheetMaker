use std::collections::BTreeMap;

use bevy_egui::egui;

use crate::ui::EditorUi;

pub fn import_window(ctx: &egui::Context, editor_ui: &mut EditorUi) {
    if !editor_ui.show_import_window {
        return;
    }

    let mut open = editor_ui.show_import_window;
    let mut close_window = false;

    egui::Window::new("Import Materials")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(500.0)
        .show(ctx, |ui| {
            ui.heading("Material Root");

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let path = editor_ui
                    .import_root
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "No folder selected".into());

                ui.add(
                    egui::TextEdit::singleline(&mut path.clone())
                        .desired_width(360.0)
                        .interactive(false),
                );

                if ui.button("Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        editor_ui.import_root = Some(path.clone());

                        match crate::importer::materials::scan(&path, &editor_ui.texture_naming) {
                            Ok(materials) => {
                                editor_ui.import_materials = materials;
                            }
                            Err(error) => {
                                eprintln!("Failed to scan material folder: {error}");
                                editor_ui.import_materials.clear();
                            }
                        }
                    }
                }
            });

            ui.separator();

            if editor_ui.import_root.is_some() {
                ui.label(format!(
                    "{} material folders found",
                    editor_ui.import_materials.len()
                ));

                ui.add_space(8.0);

                // Build a group -> materials/files preview.
                let mut groups: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

                for material in &editor_ui.import_materials {
                    for texture in &material.textures {
                        let filename = texture
                            .path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("Unknown")
                            .to_owned();

                        groups
                            .entry(texture.group.clone())
                            .or_default()
                            .push((material.name.clone(), filename));
                    }
                }

                if groups.is_empty() {
                    ui.label("No recognized textures found.");
                } else {
                    ui.label("Detected Texture Groups");

                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .max_height(250.0)
                        .show(ui, |ui| {
                            for (group, textures) in &groups {
                                egui::CollapsingHeader::new(egui::RichText::new(group).strong())
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        for (material_name, filename) in textures {
                                            ui.horizontal(|ui| {
                                                ui.add_space(12.0);

                                                ui.label(
                                                    egui::RichText::new(material_name).strong(),
                                                );

                                                ui.separator();

                                                ui.label(filename);
                                            });
                                        }
                                    });
                            }
                        });
                }
            } else {
                ui.label("Select a folder containing material folders.");
            }

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    close_window = true;
                }

                let can_import = editor_ui.import_root.is_some();

                if ui
                    .add_enabled(can_import, egui::Button::new("Import"))
                    .clicked()
                {
                    // Import materials here.
                    close_window = true;
                }
            });
        });

    if close_window {
        open = false;
    }

    editor_ui.show_import_window = open;
}
