use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    data::Atlas,
    importer::{classify_textures, material_source_from_paths},
    ui::EditorUi,
};

pub fn import_single_material_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    images: &mut Assets<Image>,
) {
    if !editor_ui.show_import_single_material_window {
        return;
    }

    let mut open = editor_ui.show_import_single_material_window;
    let mut close_window = false;

    egui::Window::new("Import Material")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(600.0)
        .show(ctx, |ui| {
            ui.heading("Material");

            ui.add_space(8.0);

            // -----------------------------------------------------------------
            // Folder
            // -----------------------------------------------------------------

            ui.horizontal(|ui| {
                ui.label("Folder:");

                let path = editor_ui
                    .single_material_root
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "No folder selected".to_string());

                ui.add(
                    egui::TextEdit::singleline(&mut path.clone())
                        .desired_width(400.0)
                        .interactive(false),
                );

                if ui.button("Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        match crate::importer::single_material::scan(&path) {
                            Ok(scan) => {
                                editor_ui.single_material_root = Some(path);
                                editor_ui.single_material_name = scan.name;

                                editor_ui
                                    .project_settings
                                    .texture_naming
                                    .merge_detected(scan.naming);

                                editor_ui.single_material_textures.clear();

                                let textures = classify_textures(
                                    scan.textures,
                                    &editor_ui.project_settings.texture_naming,
                                );

                                for texture in textures {
                                    editor_ui
                                        .single_material_textures
                                        .insert(texture.id, Some(texture.path));
                                }
                            }

                            Err(error) => {
                                eprintln!("Failed to scan material folder: {error}");

                                editor_ui.single_material_root = None;
                                editor_ui.single_material_name.clear();
                                editor_ui.single_material_textures.clear();
                            }
                        }
                    }
                }
            });

            ui.add_space(12.0);

            // -----------------------------------------------------------------
            // Name
            // -----------------------------------------------------------------

            ui.horizontal(|ui| {
                ui.label("Name:");

                ui.add(
                    egui::TextEdit::singleline(&mut editor_ui.single_material_name)
                        .desired_width(400.0),
                );
            });

            ui.add_space(12.0);

            ui.separator();

            ui.add_space(8.0);

            // -----------------------------------------------------------------
            // Textures
            // -----------------------------------------------------------------

            ui.heading("Textures");

            ui.add_space(8.0);

            if editor_ui.single_material_root.is_none() {
                ui.label(
                    egui::RichText::new("Select a material folder to detect its textures.").weak(),
                );
            } else {
                let texture_groups: Vec<(String, String)> = editor_ui
                    .project_settings
                    .texture_naming
                    .groups
                    .iter()
                    .map(|group| (group.id.clone(), group.name.clone()))
                    .collect();

                if texture_groups.is_empty() {
                    ui.label(egui::RichText::new("No texture groups have been configured.").weak());
                } else {
                    for (id, name) in texture_groups {
                        texture_file_row(ui, &name, &id, editor_ui);
                    }
                }
            }

            ui.add_space(16.0);

            ui.separator();

            ui.add_space(8.0);

            // -----------------------------------------------------------------
            // Buttons
            // -----------------------------------------------------------------

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    close_window = true;
                }

                let can_import = !editor_ui.single_material_name.trim().is_empty()
                    && editor_ui
                        .single_material_textures
                        .values()
                        .any(|path| path.is_some());

                if ui
                    .add_enabled(can_import, egui::Button::new("Import"))
                    .clicked()
                {
                    import_material(editor_ui, atlas, images);

                    close_window = true;
                }
            });
        });

    if close_window {
        open = false;
    }

    editor_ui.show_import_single_material_window = open;

    if !open {
        reset_single_material_state(editor_ui);
    }
}

fn texture_file_row(ui: &mut egui::Ui, label: &str, id: &str, editor_ui: &mut EditorUi) {
    let current_path = editor_ui
        .single_material_textures
        .get(id)
        .and_then(|path| path.as_ref())
        .cloned();

    ui.horizontal(|ui| {
        ui.label(label);

        let path_text = current_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Not found");

        ui.add(
            egui::TextEdit::singleline(&mut path_text.to_string())
                .desired_width(300.0)
                .interactive(false),
        );

        if ui.button("Browse").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("EXR", &["exr"])
                .pick_file()
            {
                editor_ui
                    .single_material_textures
                    .insert(id.to_string(), Some(path));
            }
        }

        if current_path.is_some() && ui.button("Clear").clicked() {
            editor_ui
                .single_material_textures
                .insert(id.to_string(), None);
        }
    });
}

fn import_material(editor_ui: &mut EditorUi, atlas: &mut Atlas, images: &mut Assets<Image>) {
    let name = editor_ui.single_material_name.trim().to_string();

    if name.is_empty() {
        return;
    }

    let textures = editor_ui
        .project_settings
        .texture_naming
        .groups
        .iter()
        .filter_map(|group| {
            editor_ui
                .single_material_textures
                .get(&group.id)
                .and_then(|path| path.as_ref())
                .map(|path| (group.id.clone(), path.clone()))
        })
        .collect::<Vec<_>>();

    if textures.is_empty() {
        return;
    }

    let source = material_source_from_paths(name, textures);

    let index = atlas.add_material(source);

    atlas.load_textures(
        images,
        editor_ui.project_settings.preview_texture_group.as_deref(),
    );

    editor_ui.selected_material = Some(index);
}

fn reset_single_material_state(editor_ui: &mut EditorUi) {
    editor_ui.single_material_root = None;
    editor_ui.single_material_name.clear();
    editor_ui.single_material_textures.clear();
}
