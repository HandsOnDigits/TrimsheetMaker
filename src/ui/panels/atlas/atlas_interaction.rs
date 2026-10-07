use bevy_egui::egui;

use crate::{
    data::{Atlas, AtlasMaterial},
    history::{EditorHistory, EditorSnapshot},
    mesh::MeshSceneState,
    ui::EditorUi,
};

pub fn handle_material_selection(
    response: &egui::Response,
    viewport: egui::Rect,
    editor_ui: &mut EditorUi,
    atlas: &Atlas,
) {
    if !response.clicked_by(egui::PointerButton::Primary) {
        return;
    }

    let atlas_rect = atlas_screen_rect(viewport, editor_ui, atlas);

    let Some(pointer) = response.interact_pointer_pos() else {
        return;
    };

    let Some(index) = atlas_material_at_position(pointer, atlas_rect, editor_ui, atlas) else {
        return;
    };

    editor_ui.inspected_atlas_region = Some(index);
}

pub fn handle_material_drag(
    response: &egui::Response,
    viewport: egui::Rect,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    history: &mut EditorHistory,
    mesh_scene_state: &MeshSceneState,
) {
    let atlas_rect = atlas_screen_rect(viewport, editor_ui, atlas);

    if response.drag_started_by(egui::PointerButton::Primary) {
        editor_ui.atlas_drag_snapshot = Some(EditorSnapshot::capture(
            atlas,
            &editor_ui.project_settings,
            mesh_scene_state,
        ));

        start_atlas_region_drag(response, atlas_rect, editor_ui, atlas);
    }

    if response.dragged_by(egui::PointerButton::Primary) {
        drag_atlas_region(response, atlas_rect, editor_ui, atlas);
    }

    if response.drag_stopped_by(egui::PointerButton::Primary) {
        editor_ui.dragging_atlas_region = None;
        editor_ui.drag_offset = egui::Vec2::ZERO;

        atlas.mark_bake_dirty();

        if let Some(before) = editor_ui.atlas_drag_snapshot.take() {
            history.push(before);
        }
    }
}

fn start_atlas_region_drag(
    response: &egui::Response,
    atlas_rect: egui::Rect,
    editor_ui: &mut EditorUi,
    atlas: &Atlas,
) {
    let Some(pointer) = response.interact_pointer_pos() else {
        return;
    };

    let Some(index) = atlas_material_at_position(pointer, atlas_rect, editor_ui, atlas) else {
        return;
    };

    let atlas_position = screen_to_atlas(pointer, atlas_rect, editor_ui.atlas_zoom);

    let Some(atlas_material) = atlas
        .materials
        .iter()
        .find(|material| material.index == index)
    else {
        return;
    };

    editor_ui.dragging_atlas_region = Some(index);

    editor_ui.drag_offset = egui::vec2(
        atlas_position.x - atlas_material.transform.x as f32,
        atlas_position.y - atlas_material.transform.y as f32,
    );
}

fn drag_atlas_region(
    response: &egui::Response,
    atlas_rect: egui::Rect,
    editor_ui: &EditorUi,
    atlas: &mut Atlas,
) {
    let Some(index) = editor_ui.dragging_atlas_region else {
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
        editor_ui.snap_distance,
    );

    atlas.mark_uv_dirty();
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
    atlas_rect: egui::Rect,
    editor_ui: &EditorUi,
    atlas: &Atlas,
) -> Option<usize> {
    let atlas_position = screen_to_atlas(position, atlas_rect, editor_ui.atlas_zoom);

    atlas
        .materials
        .iter()
        .rev()
        .find(|atlas_material| material_rect_in_atlas(atlas_material).contains(atlas_position))
        .map(|atlas_material| atlas_material.index)
}

pub fn material_rect_in_atlas(atlas_material: &AtlasMaterial) -> egui::Rect {
    let transform = &atlas_material.transform;

    let width = transform.rotated_width() as f32;
    let height = transform.rotated_height() as f32;

    egui::Rect::from_min_size(
        egui::pos2(transform.x as f32, transform.y as f32),
        egui::vec2(width, height),
    )
}
