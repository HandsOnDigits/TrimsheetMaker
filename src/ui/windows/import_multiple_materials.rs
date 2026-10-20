use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    data::Atlas,
    native_dialog::{NativeDialogRequest, NativeDialogState},
    ui::EditorUi,
};

use super::FloatingWindowInteraction;

pub fn import_multiple_materials_window(
    ctx: &egui::Context,
    dialog: &mut NativeDialogState,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    images: &mut Assets<Image>,
) -> bool {
    let mut open = editor_ui.show_import_multiple_materials_window;
    let mut close_window = false;

    let window_response = egui::Window::new("Import Materials")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(500.0)
        .show(ctx, |ui| {
            ui.heading("Material Root");

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let path = editor_ui
                    .multiple_materials_root
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "No folder selected".into());

                ui.add(
                    egui::TextEdit::singleline(&mut path.clone())
                        .desired_width(360.0)
                        .interactive(false),
                );

                if ui.button("Browse").clicked() {
                    dialog.start(NativeDialogRequest::MultipleMaterialsFolder);
                }
            });

            ui.separator();

            if editor_ui.multiple_materials_root.is_some() {
                ui.label(format!(
                    "{} material folders found",
                    editor_ui.import_materials.len()
                ));

                ui.add_space(8.0);

                ui.label("Detected Texture Naming");

                ui.add_space(4.0);

                for group in &editor_ui.project_settings.texture_naming.groups {
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

                        let Some(group_id) = &texture.group_id else {
                            continue;
                        };

                        groups
                            .entry(group_id.clone())
                            .or_default()
                            .push((material.name.clone(), filename));
                    }
                }

                for textures in groups.values_mut() {
                    textures.sort_by(|a, b| a.0.cmp(&b.0));
                }
            } else {
                ui.label("Select a folder containing material folders.");
            }

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    close_window = true;
                }

                let can_import = editor_ui.multiple_materials_root.is_some()
                    && !editor_ui.import_materials.is_empty();

                if ui
                    .add_enabled(can_import, egui::Button::new("Import"))
                    .clicked()
                {
                    let materials = std::mem::take(&mut editor_ui.import_materials);

                    for source in materials.into_values() {
                        atlas.add_material(source);
                    }

                    atlas.load_textures(images);

                    editor_ui.project_settings.sync_from_atlas(atlas);

                    editor_ui.multiple_materials_root = None;
                    close_window = true;
                }
            });
        });

    let owns_pointer = window_response
        .as_ref()
        .map(|response| response.owns_pointer(ctx))
        .unwrap_or(false);

    editor_ui.show_import_multiple_materials_window = open;

    owns_pointer
}
