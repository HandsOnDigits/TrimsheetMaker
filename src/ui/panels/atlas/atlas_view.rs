use bevy::{
    input::ButtonInput,
    prelude::{KeyCode, Vec2},
};
use bevy_egui::egui;

use super::atlas_interaction;

use crate::{
    data::{Atlas, AtlasMaterial, TextureNaming},
    history::EditorHistory,
    mesh::MeshSceneState,
    transform::{Rotation, UvTransform},
    ui::EditorUi,
    viewport::{MeshPreview, TransformMode},
};

pub fn atlas_view(
    ui: &mut egui::Ui,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    keyboard: &ButtonInput<KeyCode>,
    mesh_scene_state: &mut MeshSceneState,
    history: &mut EditorHistory,
    mesh_preview: &mut MeshPreview,
) {
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

        ui.separator();

        if ui.button("Reset").clicked() {
            editor_ui.atlas_zoom = 1.0;
            editor_ui.atlas_pan = egui::Vec2::ZERO;
        }

        ui.separator();

        let mut uv_mode = mesh_preview.transform_mode == TransformMode::Uv;

        if ui.checkbox(&mut uv_mode, "UV Transform").changed() {
            mesh_preview.transform_mode = if uv_mode {
                TransformMode::Uv
            } else {
                TransformMode::None
            };
        }
    });

    ui.separator();

    let available = ui.available_size();

    let (viewport_rect, response) =
        ui.allocate_exact_size(available, egui::Sense::click_and_drag());

    // Middle/right mouse button panning.
    if response.dragged_by(egui::PointerButton::Middle)
        || response.dragged_by(egui::PointerButton::Secondary)
    {
        editor_ui.atlas_pan += response.drag_delta();
    }

    // Mouse wheel scrolling / Ctrl + vertical wheel zoom.
    if response.hovered() {
        let ctrl =
            keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

        let scroll = ui.input(|input| {
            input
                .raw
                .events
                .iter()
                .fold(egui::Vec2::ZERO, |scroll, event| match event {
                    egui::Event::MouseWheel { delta, .. } => scroll + *delta,
                    _ => scroll,
                })
        });

        if ctrl && scroll.y != 0.0 {
            let zoom_factor = if scroll.y > 0.0 { 1.1 } else { 0.9 };

            editor_ui.atlas_zoom = (editor_ui.atlas_zoom * zoom_factor).clamp(0.1, 10.0);
        } else {
            editor_ui.atlas_pan += scroll * editor_ui.editor_settings.mouse_scroll_speed;
        }
    }

    atlas_interaction::handle_material_selection(&response, viewport_rect, editor_ui, atlas);
    atlas_interaction::handle_material_drag(
        &response,
        viewport_rect,
        editor_ui,
        atlas,
        history,
        mesh_scene_state,
        mesh_preview,
    );

    let painter = ui.painter_at(viewport_rect);

    draw_atlas(
        &painter,
        viewport_rect,
        editor_ui,
        atlas,
        mesh_scene_state,
        &editor_ui.project_settings.texture_naming,
    );
}

fn draw_atlas(
    painter: &egui::Painter,
    viewport: egui::Rect,
    editor_ui: &EditorUi,
    atlas: &Atlas,
    mesh_scene_state: &MeshSceneState,
    texture_naming: &TextureNaming,
) {
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

    let preview_group = editor_ui.project_settings.preview_texture_group.as_deref();

    /*
     * Draw every atlas region.
     *
     * There is no selected material. An AtlasMaterial is simply
     * an atlas region with a transform and mesh assignments.
     */
    for atlas_material in &atlas.materials {
        let rect = atlas_material_rect(atlas_material, atlas_rect, zoom);

        draw_material_region(painter, rect, atlas_material, preview_group, texture_naming);

        draw_uv_overlay(painter, rect, atlas_material, mesh_scene_state);
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
    preview_group: Option<&str>,
    texture_naming: &TextureNaming,
) {
    /*
     * Every AtlasMaterial is an atlas region.
     *
     * There is deliberately no "selected" state here.
     */
    let fill = egui::Color32::from_rgba_unmultiplied(120, 120, 120, 10);

    painter.rect_filled(rect, 0.0, fill);

    if let Some(texture_id) = preview_group.and_then(|group_id| {
        material
            .textures
            .iter()
            .find(|texture| texture_naming.texture_matches_group(&texture.path, group_id))
            .and_then(|texture| texture.egui_texture)
    }) {
        draw_rotated_texture(painter, rect, texture_id, material.transform.rotation);
    }

    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(170, 170, 170));

    painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
}

fn atlas_material_rect(material: &AtlasMaterial, atlas_rect: egui::Rect, zoom: f32) -> egui::Rect {
    let (x, y, width, height) = material.transform.atlas_rect();

    let min = atlas_rect.min + egui::vec2(x as f32 * zoom, y as f32 * zoom);

    egui::Rect::from_min_size(min, egui::vec2(width as f32 * zoom, height as f32 * zoom))
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

    let (uv0, uv1, uv2, uv3) = match rotation {
        Rotation::Deg0 => (
            egui::pos2(0.0, 0.0),
            egui::pos2(1.0, 0.0),
            egui::pos2(1.0, 1.0),
            egui::pos2(0.0, 1.0),
        ),

        Rotation::Deg90 => (
            egui::pos2(0.0, 1.0),
            egui::pos2(0.0, 0.0),
            egui::pos2(1.0, 0.0),
            egui::pos2(1.0, 1.0),
        ),

        Rotation::Deg180 => (
            egui::pos2(1.0, 1.0),
            egui::pos2(0.0, 1.0),
            egui::pos2(0.0, 0.0),
            egui::pos2(1.0, 0.0),
        ),

        Rotation::Deg270 => (
            egui::pos2(1.0, 0.0),
            egui::pos2(1.0, 1.0),
            egui::pos2(0.0, 1.0),
            egui::pos2(0.0, 0.0),
        ),
    };

    mesh.vertices.push(egui::epaint::Vertex {
        pos: min,
        uv: uv0,
        color: egui::Color32::WHITE,
    });

    mesh.vertices.push(egui::epaint::Vertex {
        pos: egui::pos2(max.x, min.y),
        uv: uv1,
        color: egui::Color32::WHITE,
    });

    mesh.vertices.push(egui::epaint::Vertex {
        pos: max,
        uv: uv2,
        color: egui::Color32::WHITE,
    });

    mesh.vertices.push(egui::epaint::Vertex {
        pos: egui::pos2(min.x, max.y),
        uv: uv3,
        color: egui::Color32::WHITE,
    });

    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);

    painter.add(egui::Shape::mesh(mesh));
}

fn draw_uv_overlay(
    painter: &egui::Painter,
    material_rect: egui::Rect,
    material: &AtlasMaterial,
    mesh_scene_state: &MeshSceneState,
) {
    let stroke = egui::Stroke::new(
        1.0,
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180),
    );

    for assignment in &material.mesh_assignments {
        let Some(mesh) = mesh_scene_state.get(assignment.mesh_id) else {
            continue;
        };

        let Some(object) = mesh.scene.objects.get(assignment.object_index) else {
            continue;
        };

        let Some(primitive) = object.primitives.get(assignment.primitive_index) else {
            continue;
        };

        let uvs = &primitive.uvs;
        let indices = &primitive.indices;

        let pivot = UvTransform::uv_center(uvs);

        for triangle in indices.chunks_exact(3) {
            let ia = triangle[0] as usize;
            let ib = triangle[1] as usize;
            let ic = triangle[2] as usize;

            let Some(&a) = uvs.get(ia) else {
                continue;
            };

            let Some(&b) = uvs.get(ib) else {
                continue;
            };

            let Some(&c) = uvs.get(ic) else {
                continue;
            };

            let a = transform_uv(
                a,
                pivot,
                assignment.uv_transform,
                material.transform.rotation,
            );

            let b = transform_uv(
                b,
                pivot,
                assignment.uv_transform,
                material.transform.rotation,
            );

            let c = transform_uv(
                c,
                pivot,
                assignment.uv_transform,
                material.transform.rotation,
            );

            let a = material_uv_to_screen(a, material_rect);
            let b = material_uv_to_screen(b, material_rect);
            let c = material_uv_to_screen(c, material_rect);

            painter.line_segment([a, b], stroke);
            painter.line_segment([b, c], stroke);
            painter.line_segment([c, a], stroke);
        }
    }
}

fn material_uv_to_screen(uv: [f32; 2], material_rect: egui::Rect) -> egui::Pos2 {
    egui::pos2(
        material_rect.left() + uv[0] * material_rect.width(),
        material_rect.bottom() - uv[1] * material_rect.height(),
    )
}

fn transform_uv(
    uv: [f32; 2],
    pivot: Vec2,
    uv_transform: UvTransform,
    rotation: Rotation,
) -> [f32; 2] {
    let uv = uv_transform.apply(Vec2::new(uv[0], uv[1]), pivot);

    transform_material_uv([uv.x, uv.y], rotation)
}

fn transform_material_uv(uv: [f32; 2], rotation: Rotation) -> [f32; 2] {
    match rotation {
        Rotation::Deg0 => uv,
        Rotation::Deg90 => [uv[1], 1.0 - uv[0]],
        Rotation::Deg180 => [1.0 - uv[0], 1.0 - uv[1]],
        Rotation::Deg270 => [1.0 - uv[1], uv[0]],
    }
}
