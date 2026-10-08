use bevy::math::Vec2;
use bevy_egui::egui;

use crate::{
    data::{Atlas, AtlasMaterial},
    history::{EditorHistory, EditorSnapshot},
    mesh::MeshSceneState,
    ui::EditorUi,
    viewport::{MeshPreview, TransformMode},
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
    mesh_scene_state: &mut MeshSceneState,
    mesh_preview: &mut MeshPreview,
) {
    let atlas_rect = atlas_screen_rect(viewport, editor_ui, atlas);

    if response.drag_started_by(egui::PointerButton::Primary) {
        // Select the region immediately when dragging starts.
        if let Some(pointer) = response.interact_pointer_pos() {
            if let Some(index) = atlas_material_at_position(pointer, atlas_rect, editor_ui, atlas) {
                editor_ui.inspected_atlas_region = Some(index);
            }
        }

        editor_ui.atlas_drag_snapshot = Some(EditorSnapshot::capture(
            atlas,
            &editor_ui.project_settings,
            mesh_scene_state,
        ));

        if mesh_preview.transform_mode == TransformMode::Uv {
            start_uv_drag(response, atlas_rect, editor_ui, atlas, mesh_scene_state);
        } else {
            start_atlas_region_drag(response, atlas_rect, editor_ui, atlas);
        }
    }

    if response.dragged_by(egui::PointerButton::Primary) {
        if mesh_preview.transform_mode == TransformMode::Uv {
            drag_uv(response, editor_ui, atlas, mesh_scene_state);
        } else {
            drag_atlas_region(response, atlas_rect, editor_ui, atlas);
        }
    }

    if response.drag_stopped_by(egui::PointerButton::Primary) {
        editor_ui.dragging_atlas_region = None;
        editor_ui.drag_offset = egui::Vec2::ZERO;
        editor_ui.atlas_uv_drag_start = None;

        if mesh_preview.transform_mode != TransformMode::Uv {
            atlas.mark_bake_dirty();
        }

        if let Some(before) = editor_ui.atlas_drag_snapshot.take() {
            history.push(before);
        }
    }
}

fn start_uv_drag(
    response: &egui::Response,
    atlas_rect: egui::Rect,
    editor_ui: &mut EditorUi,
    atlas: &Atlas,
    mesh_scene_state: &MeshSceneState,
) {
    let Some(pointer) = response.interact_pointer_pos() else {
        return;
    };

    let Some(index) = atlas_material_at_position(pointer, atlas_rect, editor_ui, atlas) else {
        return;
    };

    editor_ui.inspected_atlas_region = Some(index);

    let Some(material) = atlas.materials.iter().find(|m| m.index == index) else {
        return;
    };

    let mut starts = Vec::new();

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

        starts.push((
            assignment.mesh_id,
            assignment.object_index,
            assignment.primitive_index,
            primitive.uv_transform,
        ));
    }

    editor_ui.atlas_uv_drag_start = Some(starts);
}

fn drag_uv(
    response: &egui::Response,
    editor_ui: &EditorUi,
    atlas: &Atlas,
    mesh_scene_state: &mut MeshSceneState,
) {
    let Some(index) = editor_ui.inspected_atlas_region else {
        return;
    };

    let Some(starts) = &editor_ui.atlas_uv_drag_start else {
        return;
    };

    let Some(material) = atlas.materials.iter().find(|m| m.index == index) else {
        return;
    };

    let Some(pointer) = response.interact_pointer_pos() else {
        return;
    };

    let Some(start_pointer) = response.ctx.input(|input| input.pointer.press_origin()) else {
        return;
    };

    let delta = pointer - start_pointer;

    let atlas_zoom = editor_ui.atlas_zoom.max(0.0001);

    let width = material.transform.rotated_width().max(1) as f32;
    let height = material.transform.rotated_height().max(1) as f32;

    let uv_delta = Vec2::new(delta.x / atlas_zoom / width, -delta.y / atlas_zoom / height);

    for &(mesh_id, object_index, primitive_index, start) in starts {
        let Some(mesh) = mesh_scene_state.get_mut(mesh_id) else {
            continue;
        };

        let Some(object) = mesh.scene.objects.get_mut(object_index) else {
            continue;
        };

        let Some(primitive) = object.primitives.get_mut(primitive_index) else {
            continue;
        };

        primitive.uv_transform.translation = start.translation + uv_delta;
    }

    mesh_scene_state.mark_uv_dirty();
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
