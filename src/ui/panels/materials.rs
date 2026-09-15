use std::collections::BTreeSet;

use bevy_egui::egui;

use super::EditorUi;

use crate::data::{Atlas, AtlasMaterial};

pub fn materials_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.heading("Materials");

    ui.horizontal(|ui| {
        ui.label(format!("{} materials", atlas.materials.len()));
    });

    ui.separator();

    // Collect all texture groups currently available in the atlas.
    let groups: BTreeSet<String> = atlas
        .materials
        .iter()
        .flat_map(|material| {
            material
                .textures
                .iter()
                .map(|texture| texture.group.clone())
        })
        .collect();

    ui.label("Preview Map");

    let selected_group = editor_ui
        .preview_texture_group
        .clone()
        .unwrap_or_else(|| "Select texture map...".to_string());

    egui::ComboBox::from_id_salt("preview_texture_group_editor")
        .selected_text(selected_group)
        .show_ui(ui, |ui| {
            for group in &groups {
                ui.selectable_value(
                    &mut editor_ui.preview_texture_group,
                    Some(group.clone()),
                    group,
                );
            }
        });

    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("materials_scroll")
        .show(ui, |ui| {
            let mut clicked = None;

            for material in &atlas.materials {
                let selected = editor_ui.selected_material == Some(material.index);

                if material_row(ui, material, selected).clicked() {
                    clicked = Some(material.index);
                }
            }

            if let Some(index) = clicked {
                editor_ui.selected_material = Some(index);
            }
        });

    ui.separator();

    if ui
        .add_sized(
            [ui.available_width(), 32.0],
            egui::Button::new("＋ Add Material"),
        )
        .clicked()
    {
        let index = atlas.add_placeholder();
        editor_ui.selected_material = Some(index);
    }
}

fn material_row(ui: &mut egui::Ui, material: &AtlasMaterial, selected: bool) -> egui::Response {
    let height = 48.0;
    let width = ui.available_width();

    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    if selected {
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

    ui.painter().text(
        thumbnail_rect.center(),
        egui::Align2::CENTER_CENTER,
        material.index.to_string(),
        egui::FontId::proportional(14.0),
        egui::Color32::WHITE,
    );

    let text_pos = rect.left_top() + egui::vec2(52.0, 7.0);

    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_TOP,
        &material.name,
        egui::FontId::proportional(14.0),
        ui.visuals().text_color(),
    );

    let texture_count = material.textures.len();

    ui.painter().text(
        text_pos + egui::vec2(0.0, 19.0),
        egui::Align2::LEFT_TOP,
        format!("#{}  {} textures", material.index, texture_count),
        egui::FontId::proportional(11.0),
        ui.visuals().weak_text_color(),
    );

    response
}
