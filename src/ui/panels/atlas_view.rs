use bevy_egui::egui;

use super::{Atlas, AtlasMaterial, EditorUi};

pub fn atlas_view(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.horizontal(|ui| {
        ui.heading("Atlas");

        ui.separator();

        ui.label(format!("{} × {} px", atlas.width, atlas.height));

        ui.separator();

        if ui.button("−").clicked() {
            editor_ui.atlas_zoom = (editor_ui.atlas_zoom * 0.9).max(0.1);
        }

        ui.label(format!("{:.0}%", editor_ui.atlas_zoom * 100.0));

        if ui.button("+").clicked() {
            editor_ui.atlas_zoom = (editor_ui.atlas_zoom * 1.1).min(10.0);
        }

        if ui.button("Reset").clicked() {
            editor_ui.atlas_zoom = 1.0;
            editor_ui.atlas_pan = egui::Vec2::ZERO;
        }
    });

    ui.separator();

    let available = ui.available_size();

    let (viewport_rect, response) =
        ui.allocate_exact_size(available, egui::Sense::click_and_drag());

    if response.dragged_by(egui::PointerButton::Middle)
        || response.dragged_by(egui::PointerButton::Secondary)
    {
        editor_ui.atlas_pan += response.drag_delta();
    }

    let painter = ui.painter_at(viewport_rect);

    draw_atlas(&painter, viewport_rect, editor_ui, atlas);

    if response.clicked() {
        if let Some(pointer) = response.interact_pointer_pos() {
            editor_ui.selected_material =
                atlas_material_at_position(pointer, viewport_rect, editor_ui, atlas);
        }
    }
}

fn draw_atlas(painter: &egui::Painter, viewport: egui::Rect, editor_ui: &EditorUi, atlas: &Atlas) {
    let zoom = editor_ui.atlas_zoom;

    let atlas_size = egui::vec2(atlas.width as f32 * zoom, atlas.height as f32 * zoom);

    let center = viewport.center() + editor_ui.atlas_pan;

    let atlas_rect = egui::Rect::from_center_size(center, atlas_size);

    draw_checkerboard(painter, atlas_rect, zoom);

    painter.rect_stroke(
        atlas_rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(120)),
        egui::StrokeKind::Outside,
    );

    for material in &atlas.materials {
        let rect = atlas_material_rect(material, atlas_rect, zoom);

        let selected = editor_ui.selected_material == Some(material.index);

        draw_material_region(painter, rect, material, selected);
    }
}

fn draw_checkerboard(painter: &egui::Painter, rect: egui::Rect, zoom: f32) {
    let checker_size = (16.0 * zoom).max(4.0);

    let columns = (rect.width() / checker_size).ceil() as i32;

    let rows = (rect.height() / checker_size).ceil() as i32;

    let light = egui::Color32::from_gray(58);
    let dark = egui::Color32::from_gray(48);

    for y in 0..rows {
        for x in 0..columns {
            let min = egui::pos2(
                rect.left() + x as f32 * checker_size,
                rect.top() + y as f32 * checker_size,
            );

            let max = egui::pos2(min.x + checker_size, min.y + checker_size);

            let tile = egui::Rect::from_min_max(min, max).intersect(rect);

            let color = if (x + y) % 2 == 0 { light } else { dark };

            painter.rect_filled(tile, 0.0, color);
        }
    }
}

fn draw_material_region(
    painter: &egui::Painter,
    rect: egui::Rect,
    material: &AtlasMaterial,
    selected: bool,
) {
    let fill = if selected {
        egui::Color32::from_rgba_unmultiplied(80, 140, 255, 100)
    } else {
        egui::Color32::from_rgba_unmultiplied(120, 120, 120, 45)
    };

    painter.rect_filled(rect, 0.0, fill);

    let stroke = if selected {
        egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 170, 255))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(170, 170, 170))
    };

    painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        material.index.to_string(),
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );
}

fn atlas_material_rect(material: &AtlasMaterial, atlas_rect: egui::Rect, zoom: f32) -> egui::Rect {
    let min = egui::pos2(
        atlas_rect.left() + material.x as f32 * zoom,
        atlas_rect.top() + material.y as f32 * zoom,
    );

    let size = egui::vec2(material.width as f32 * zoom, material.height as f32 * zoom);

    egui::Rect::from_min_size(min, size)
}

fn atlas_material_at_position(
    position: egui::Pos2,
    viewport: egui::Rect,
    editor_ui: &EditorUi,
    atlas: &Atlas,
) -> Option<usize> {
    let zoom = editor_ui.atlas_zoom;

    let atlas_size = egui::vec2(atlas.width as f32 * zoom, atlas.height as f32 * zoom);

    let center = viewport.center() + editor_ui.atlas_pan;

    let atlas_rect = egui::Rect::from_center_size(center, atlas_size);

    if !atlas_rect.contains(position) {
        return None;
    }

    let atlas_x = ((position.x - atlas_rect.left()) / zoom) as u32;

    let atlas_y = ((position.y - atlas_rect.top()) / zoom) as u32;

    atlas
        .materials
        .iter()
        .rev()
        .find(|material| {
            atlas_x >= material.x
                && atlas_x < material.x + material.width
                && atlas_y >= material.y
                && atlas_y < material.y + material.height
        })
        .map(|material| material.index)
}
