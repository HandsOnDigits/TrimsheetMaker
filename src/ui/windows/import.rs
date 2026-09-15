use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{data::Atlas, ui::EditorUi};

pub fn import_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    images: &mut Assets<Image>,
) {
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
                        match crate::importer::materials::scan(&path) {
                            Ok(scan) => {
                                editor_ui.import_root = Some(path);
                                editor_ui.import_materials = scan.materials;
                                editor_ui.texture_naming = scan.naming;
                            }
                            Err(error) => {
                                eprintln!("Failed to scan material folder: {error}");

                                editor_ui.import_root = None;
                                editor_ui.import_materials.clear();
                                editor_ui.texture_naming = crate::data::TextureNaming::default();
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

                ui.label("Detected Texture Naming");

                ui.add_space(4.0);

                for group in &editor_ui.texture_naming.groups {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&group.name).strong());

                        ui.separator();

                        ui.label(group.suffixes.join(", "));
                    });
                }

                ui.add_space(8.0);

                let mut groups: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

                for material in editor_ui.import_materials.values() {
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

                for textures in groups.values_mut() {
                    textures.sort_by(|a, b| a.0.cmp(&b.0));
                }

                if editor_ui.preview_texture_group.is_none() {
                    ui.add_space(8.0);

                    ui.label("Default Preview Map");

                    ui.add_space(4.0);

                    let available_groups: Vec<String> = editor_ui
                        .texture_naming
                        .groups
                        .iter()
                        .map(|group| group.name.clone())
                        .collect();

                    if available_groups.is_empty() {
                        ui.label("No texture groups detected.");
                    } else {
                        if editor_ui.preview_texture_group.is_none() && !groups.is_empty() {
                            ui.separator();
                        
                            ui.label(egui::RichText::new("Default Preview Map").strong());
                        
                            ui.add_space(4.0);
                        
                            let selected_preview_group = editor_ui
                                .preview_texture_group
                                .clone()
                                .unwrap_or_else(|| "Select a texture map...".to_string());
                        
                            egui::ComboBox::from_id_salt("preview_texture_group")
                                .selected_text(selected_preview_group)
                                .show_ui(ui, |ui| {
                                    for group in groups.keys() {
                                        ui.selectable_value(
                                            &mut editor_ui.preview_texture_group,
                                            Some(group.clone()),
                                            group,
                                        );
                                    }
                                });
                        
                            ui.add_space(4.0);
                        
                            if editor_ui.preview_texture_group.is_none() {
                                ui.label(
                                    egui::RichText::new("Select which texture map should be displayed in the atlas.")
                                        .weak(),
                                );
                            }
                        }
                    }
                }

                if groups.is_empty() {
                    ui.label("No recognized textures found.");
                } else {
                    ui.label("Detected Textures");

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

                let can_import =
                    editor_ui.import_root.is_some() && !editor_ui.import_materials.is_empty();

                if ui
                    .add_enabled(can_import, egui::Button::new("Import"))
                    .clicked()
                {
                    let materials = std::mem::take(&mut editor_ui.import_materials);

                    for source in materials.into_values() {
                        atlas.add_material(source);
                    }

                    atlas.load_textures(images);

                    editor_ui.import_root = None;
                    close_window = true;
                }
            });
        });

    if close_window {
        open = false;
    }

    editor_ui.show_import_window = open;
}
