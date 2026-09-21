use bevy_egui::egui;

use crate::{
    data::{Atlas, AtlasMaterial},
    ui::EditorUi,
};

pub fn handle_material_drag(
    response: &egui::Response,
    viewport: egui::Rect,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
) {
    let atlas_rect = atlas_screen_rect(viewport, editor_ui, atlas);

    if response.drag_started_by(egui::PointerButton::Primary) {
        start_material_drag(response, viewport, atlas_rect, editor_ui, atlas);
    }

    if response.dragged_by(egui::PointerButton::Primary) {
        drag_material(response, atlas_rect, editor_ui, atlas);
    }

    if response.drag_stopped_by(egui::PointerButton::Primary) {
        editor_ui.dragging_material = None;
    }
}

fn start_material_drag(
    response: &egui::Response,
    viewport: egui::Rect,
    atlas_rect: egui::Rect,
    editor_ui: &mut EditorUi,
    atlas: &Atlas,
) {
    let Some(pointer) = response.interact_pointer_pos() else {
        return;
    };

    let Some(index) = atlas_material_at_position(pointer, viewport, editor_ui, atlas) else {
        return;
    };

    editor_ui.selected_material = Some(index);
    editor_ui.dragging_material = Some(index);

    let atlas_position = screen_to_atlas(pointer, atlas_rect, editor_ui.atlas_zoom);

    if let Some(material) = atlas.materials.iter().find(|m| m.index == index) {
        editor_ui.drag_offset = egui::vec2(
            atlas_position.x - material.transform.x as f32,
            atlas_position.y - material.transform.y as f32,
        );
    }
}

fn drag_material(
    response: &egui::Response,
    atlas_rect: egui::Rect,
    editor_ui: &EditorUi,
    atlas: &mut Atlas,
) {
    let Some(index) = editor_ui.dragging_material else {
        return;
    };

    let Some(pointer) = response.interact_pointer_pos() else {
        return;
    };

    let atlas_position = screen_to_atlas(pointer, atlas_rect, editor_ui.atlas_zoom);

    let desired_x = atlas_position.x - editor_ui.drag_offset.x;

    let desired_y = atlas_position.y - editor_ui.drag_offset.y;

    super::atlas_snapping::move_material_with_snapping(
        atlas,
        index,
        desired_x,
        desired_y,
        editor_ui.snap_margin,
        editor_ui.snap_distance,
    );
}

fn atlas_screen_rect(viewport: egui::Rect, editor_ui: &EditorUi, atlas: &Atlas) -> egui::Rect {
    let size = egui::vec2(
        atlas.width as f32 * editor_ui.atlas_zoom,
        atlas.height as f32 * editor_ui.atlas_zoom,
    );

    egui::Rect::from_center_size(viewport.center() + editor_ui.atlas_pan, size)
}

fn screen_to_atlas(position: egui::Pos2, atlas_rect: egui::Rect, zoom: f32) -> egui::Pos2 {
    egui::pos2(
        (position.x - atlas_rect.left()) / zoom,
        (position.y - atlas_rect.top()) / zoom,
    )
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

fn material_rect_in_atlas(material: &AtlasMaterial) -> egui::Rect {
    let transform = &material.transform;

    let center_x = transform.x as f32 + transform.width as f32 * 0.5;

    let center_y = transform.y as f32 + transform.height as f32 * 0.5;

    let width = transform.rotated_width() as f32;

    let height = transform.rotated_height() as f32;

    egui::Rect::from_center_size(egui::pos2(center_x, center_y), egui::vec2(width, height))
}
