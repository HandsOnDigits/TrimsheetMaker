use bevy_egui::egui;

use super::EditorUi;

use crate::data::{Atlas, AtlasMaterial, TextureNaming};

pub fn materials_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.heading("Atlas Regions");

    ui.horizontal(|ui| {
        ui.label(format!("{} regions", atlas.materials.len()));
    });

    ui.separator();

    ui.label("Preview Map");

    let selected_group_name = editor_ui
        .project_settings
        .preview_texture_group
        .as_deref()
        .and_then(|id| {
            editor_ui
                .project_settings
                .texture_naming
                .groups
                .iter()
                .find(|group| group.id == id)
                .map(|group| group.name.as_str())
        })
        .unwrap_or("Select texture map...")
        .to_string();

    egui::ComboBox::from_id_salt("preview_texture_group_editor")
        .selected_text(selected_group_name)
        .show_ui(ui, |ui| {
            for group in &editor_ui.project_settings.texture_naming.groups {
                ui.selectable_value(
                    &mut editor_ui.project_settings.preview_texture_group,
                    Some(group.id.clone()),
                    &group.name,
                );
            }
        });

    ui.separator();

    ui.label(egui::RichText::new("Texture Groups").strong());

    let mut remove_group = None;
    let mut naming_changed = false;

    for (index, group) in editor_ui
        .project_settings
        .texture_naming
        .groups
        .iter_mut()
        .enumerate()
    {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name");

                ui.add(egui::TextEdit::singleline(&mut group.name).desired_width(100.0));

                if ui.button("Remove").clicked() {
                    remove_group = Some(index);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Suffixes");

                let input = editor_ui
                    .suffix_inputs
                    .entry(group.id.clone())
                    .or_insert_with(|| group.suffixes.join(", "));

                let response = ui.add(
                    egui::TextEdit::singleline(input)
                        .desired_width(180.0)
                        .hint_text("_base_color, _basecolor"),
                );

                if response.lost_focus() {
                    group.suffixes = input
                        .split(',')
                        .map(str::trim)
                        .filter(|suffix| !suffix.is_empty())
                        .map(|suffix| {
                            if suffix.starts_with('_') {
                                suffix.to_ascii_lowercase()
                            } else {
                                format!("_{}", suffix.to_ascii_lowercase())
                            }
                        })
                        .collect();

                    naming_changed = true;
                }
            });
        });

        ui.add_space(4.0);
    }

    if naming_changed {
        editor_ui
            .project_settings
            .texture_naming
            .reclassify_materials(&mut editor_ui.import_materials);

        editor_ui
            .project_settings
            .texture_naming
            .reclassify_atlas(atlas);
    }

    if let Some(index) = remove_group {
        let removed_id = editor_ui.project_settings.texture_naming.groups[index]
            .id
            .clone();

        editor_ui
            .project_settings
            .texture_naming
            .groups
            .remove(index);

        editor_ui.suffix_inputs.remove(&removed_id);

        if editor_ui.project_settings.preview_texture_group.as_deref() == Some(removed_id.as_str())
        {
            editor_ui.project_settings.preview_texture_group = None;
        }

        editor_ui
            .project_settings
            .texture_naming
            .reclassify_atlas(atlas);

        editor_ui
            .project_settings
            .texture_naming
            .reclassify_materials(&mut editor_ui.import_materials);
    }

    if ui.button("+ Add Group").clicked() {
        editor_ui
            .project_settings
            .texture_naming
            .groups
            .push(crate::data::TextureGroup {
                id: uuid::Uuid::new_v4().to_string(),
                name: "New Group".to_string(),
                suffixes: vec!["_new".to_string()],
                rendering: Default::default(),
            });
    }

    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("materials_scroll")
        .show(ui, |ui| {
            let mut inspected = None;

            for atlas_material in &atlas.materials {
                let active = editor_ui.inspected_atlas_region == Some(atlas_material.index);

                if material_row(
                    ui,
                    atlas_material,
                    active,
                    editor_ui.project_settings.preview_texture_group.as_deref(),
                    &editor_ui.project_settings.texture_naming,
                )
                .clicked()
                {
                    inspected = Some(atlas_material.index);
                }
            }

            if let Some(index) = inspected {
                editor_ui.inspected_atlas_region = Some(index);
            }
        });

    ui.separator();

    if ui
        .add_sized(
            [ui.available_width(), 32.0],
            egui::Button::new("+ Add Atlas Region"),
        )
        .clicked()
    {
        let index = atlas.add_placeholder();

        editor_ui.inspected_atlas_region = Some(index);
    }
}

fn material_row(
    ui: &mut egui::Ui,
    material: &AtlasMaterial,
    active: bool,
    preview_texture_group: Option<&str>,
    texture_naming: &TextureNaming,
) -> egui::Response {
    let height = 48.0;
    let width = ui.available_width();

    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    if active {
        ui.painter()
            .rect_filled(rect, 4.0, ui.visuals().selection.bg_fill);
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
    }

    let thumbnail_rect = egui::Rect::from_min_size(
        rect.left_top() + egui::vec2(6.0, 6.0),
        egui::vec2(36.0, 36.0),
    );

    ui.painter()
        .rect_filled(thumbnail_rect, 3.0, egui::Color32::from_gray(55));

    let preview_texture = preview_texture_group.and_then(|group_id| {
        material
            .textures
            .iter()
            .find(|texture| texture_naming.texture_matches_group(&texture.path, group_id))
            .and_then(|texture| texture.egui_texture)
    });

    if let Some(texture_id) = preview_texture {
        ui.painter().image(
            texture_id,
            thumbnail_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        ui.painter().text(
            thumbnail_rect.center(),
            egui::Align2::CENTER_CENTER,
            "—",
            egui::FontId::proportional(14.0),
            egui::Color32::WHITE,
        );
    }

    let text_pos = rect.left_top() + egui::vec2(52.0, 7.0);

    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_TOP,
        &material.name,
        egui::FontId::proportional(14.0),
        ui.visuals().text_color(),
    );

    response
}
