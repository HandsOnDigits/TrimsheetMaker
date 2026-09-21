use super::EditorUi;
use bevy_egui::egui;

use crate::{
    data::{Atlas, AtlasMaterial},
    transform::Rotation,
};

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

    let preview_group = editor_ui.preview_texture_group.as_deref();

    for material in &atlas.materials {
        let rect = atlas_material_rect(material, atlas_rect, zoom);
        let selected = editor_ui.selected_material == Some(material.index);

        // Always draw the material region; preview images render if egui_texture exists
        draw_material_region(painter, rect, material, selected, preview_group);
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
    preview_group: Option<&str>,
) {
    let fill = if selected {
        egui::Color32::from_rgba_unmultiplied(80, 140, 255, 40)
    } else {
        egui::Color32::from_rgba_unmultiplied(120, 120, 120, 10)
    };

    painter.rect_filled(rect, 0.0, fill);

    if let Some(texture_id) = preview_group.and_then(|group| {
        material
            .textures
            .iter()
            .find(|texture| texture.id.eq_ignore_ascii_case(group))
            .and_then(|texture| texture.egui_texture)
    }) {
        draw_rotated_texture(painter, rect, texture_id, material.transform.rotation);
    }

    let stroke = if selected {
        egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 170, 255))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(170, 170, 170))
    };

    painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
}

fn draw_rotated_texture(
    painter: &egui::Painter,
    rect: egui::Rect,
    texture_id: egui::TextureId,
    rotation: Rotation,
) {
    let mut mesh = egui::Mesh::with_texture(texture_id);

    let min = rect.min;
    let max = rect.max;

    let top_left = egui::pos2(min.x, min.y);
    let top_right = egui::pos2(max.x, min.y);
    let bottom_right = egui::pos2(max.x, max.y);
    let bottom_left = egui::pos2(min.x, max.y);

    match rotation {
        Rotation::Deg0 => {
            mesh.vertices.extend([
                egui::epaint::Vertex {
                    pos: top_left,
                    uv: egui::pos2(0.0, 0.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: top_right,
                    uv: egui::pos2(1.0, 0.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: bottom_right,
                    uv: egui::pos2(1.0, 1.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: bottom_left,
                    uv: egui::pos2(0.0, 1.0),
                    color: egui::Color32::WHITE,
                },
            ]);
        }

        Rotation::Deg90 => {
            mesh.vertices.extend([
                egui::epaint::Vertex {
                    pos: top_left,
                    uv: egui::pos2(0.0, 1.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: top_right,
                    uv: egui::pos2(0.0, 0.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: bottom_right,
                    uv: egui::pos2(1.0, 0.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: bottom_left,
                    uv: egui::pos2(1.0, 1.0),
                    color: egui::Color32::WHITE,
                },
            ]);
        }

        Rotation::Deg180 => {
            mesh.vertices.extend([
                egui::epaint::Vertex {
                    pos: top_left,
                    uv: egui::pos2(1.0, 1.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: top_right,
                    uv: egui::pos2(0.0, 1.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: bottom_right,
                    uv: egui::pos2(0.0, 0.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: bottom_left,
                    uv: egui::pos2(1.0, 0.0),
                    color: egui::Color32::WHITE,
                },
            ]);
        }

        Rotation::Deg270 => {
            mesh.vertices.extend([
                egui::epaint::Vertex {
                    pos: top_left,
                    uv: egui::pos2(1.0, 0.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: top_right,
                    uv: egui::pos2(1.0, 1.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: bottom_right,
                    uv: egui::pos2(0.0, 1.0),
                    color: egui::Color32::WHITE,
                },
                egui::epaint::Vertex {
                    pos: bottom_left,
                    uv: egui::pos2(0.0, 0.0),
                    color: egui::Color32::WHITE,
                },
            ]);
        }
    }

    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);

    painter.add(egui::Shape::mesh(mesh));
}

fn atlas_material_rect(material: &AtlasMaterial, atlas_rect: egui::Rect, zoom: f32) -> egui::Rect {
    let (x, y, width, height) = material.transform.atlas_rect();

    let min = atlas_rect.min + egui::vec2(x as f32 * zoom, y as f32 * zoom);

    egui::Rect::from_min_size(min, egui::vec2(width as f32 * zoom, height as f32 * zoom))
}

fn material_rect_in_atlas(material: &AtlasMaterial) -> egui::Rect {
    let transform = &material.transform;

    let center_x = transform.x as f32 + transform.width as f32 * 0.5;
    let center_y = transform.y as f32 + transform.height as f32 * 0.5;

    let width = transform.rotated_width() as f32;
    let height = transform.rotated_height() as f32;

    egui::Rect::from_center_size(egui::pos2(center_x, center_y), egui::vec2(width, height))
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

    let atlas_position = egui::pos2(
        (position.x - atlas_rect.left()) / zoom,
        (position.y - atlas_rect.top()) / zoom,
    );

    atlas
        .materials
        .iter()
        .rev()
        .find(|material| {
            let has_preview_texture = editor_ui
                .preview_texture_group
                .as_deref()
                .map(|group| {
                    material
                        .textures
                        .iter()
                        .any(|texture| texture.id.eq_ignore_ascii_case(group))
                })
                .unwrap_or(false);

            has_preview_texture && material_rect_in_atlas(material).contains(atlas_position)
        })
        .map(|material| material.index)
}
