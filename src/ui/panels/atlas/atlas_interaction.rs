use bevy::math::Vec2;
use bevy_egui::egui;

use crate::{
    data::{Atlas, AtlasMaterial},
    history::{EditorHistory, EditorSnapshot},
    mesh::MeshSceneState,
    transform::UvTransform,
    ui::EditorUi,
    viewport::{MeshPreview, TransformMode},
};

pub fn handle_material_selection(
    response: &egui::Response,
    viewport: egui::Rect,
    editor_ui: &mut EditorUi,
    atlas: &Atlas,
    mesh_scene_state: &MeshSceneState,
    mesh_preview: &mut MeshPreview,
) {
    if !response.clicked_by(egui::PointerButton::Primary) {
        return;
    }

    let atlas_rect = atlas_screen_rect(viewport, editor_ui, atlas);

    let Some(pointer) = response.interact_pointer_pos() else {
        return;
    };

    if mesh_preview.transform_mode == TransformMode::Uv {
        if let Some((material_index, assignment)) =
            uv_assignment_at_position(pointer, atlas_rect, editor_ui, atlas, mesh_scene_state)
        {
            editor_ui.inspected_atlas_region = Some(material_index);
            editor_ui.selected_uv_assignment = Some(assignment);

            mesh_preview.selected_primitive = Some(crate::viewport::MeshPrimitiveSelection {
                mesh_id: assignment.0,
                object_index: assignment.1,
                primitive_index: assignment.2,
            });
        } else {
            editor_ui.selected_uv_assignment = None;
            mesh_preview.selected_primitive = None;
        }

        return;
    }

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
            drag_uv(response, editor_ui, atlas);
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

    let Some((material_index, (mesh_id, object_index, primitive_index))) =
        uv_assignment_at_position(pointer, atlas_rect, editor_ui, atlas, mesh_scene_state)
    else {
        return;
    };

    let Some(material) = atlas
        .materials
        .iter()
        .find(|material| material.index == material_index)
    else {
        return;
    };

    let Some(assignment) = material.mesh_assignments.iter().find(|assignment| {
        assignment.mesh_id == mesh_id
            && assignment.object_index == object_index
            && assignment.primitive_index == primitive_index
    }) else {
        return;
    };

    editor_ui.inspected_atlas_region = Some(material_index);
    editor_ui.selected_uv_assignment = Some((mesh_id, object_index, primitive_index));

    editor_ui.atlas_uv_drag_start = Some(vec![(
        mesh_id,
        object_index,
        primitive_index,
        assignment.uv_transform,
    )]);
}

fn drag_uv(response: &egui::Response, editor_ui: &EditorUi, atlas: &mut Atlas) {
    let Some(index) = editor_ui.inspected_atlas_region else {
        return;
    };

    let Some(starts) = &editor_ui.atlas_uv_drag_start else {
        return;
    };

    let Some(pointer) = response.interact_pointer_pos() else {
        return;
    };

    let Some(start_pointer) = response.ctx.input(|input| input.pointer.press_origin()) else {
        return;
    };

    let Some(material) = atlas
        .materials
        .iter()
        .find(|material| material.index == index)
    else {
        return;
    };

    let delta = pointer - start_pointer;

    let atlas_zoom = editor_ui.atlas_zoom.max(0.0001);

    let width = material.transform.width.max(1) as f32;
    let height = material.transform.height.max(1) as f32;

    let uv_delta = Vec2::new(delta.x / atlas_zoom / width, -delta.y / atlas_zoom / height);

    let Some(material) = atlas
        .materials
        .iter_mut()
        .find(|material| material.index == index)
    else {
        return;
    };

    for &(mesh_id, object_index, primitive_index, start) in starts {
        let Some(assignment) = material.mesh_assignments.iter_mut().find(|assignment| {
            assignment.mesh_id == mesh_id
                && assignment.object_index == object_index
                && assignment.primitive_index == primitive_index
        }) else {
            continue;
        };

        assignment.uv_transform.translation = start.translation + uv_delta;
    }

    atlas.mark_uv_dirty();
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

    editor_ui.inspected_atlas_region = Some(index);

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

fn uv_assignment_at_position(
    pointer: egui::Pos2,
    atlas_rect: egui::Rect,
    editor_ui: &EditorUi,
    atlas: &Atlas,
    mesh_scene_state: &MeshSceneState,
) -> Option<(usize, (u64, usize, usize))> {
    for material in atlas.materials.iter().rev() {
        for assignment in material.mesh_assignments.iter().rev() {
            let Some(mesh) = mesh_scene_state.get(assignment.mesh_id) else {
                continue;
            };

            let Some(object) = mesh.scene.objects.get(assignment.object_index) else {
                continue;
            };

            let Some(primitive) = object.primitives.get(assignment.primitive_index) else {
                continue;
            };

            if uv_assignment_contains_pointer(
                pointer,
                atlas_rect,
                editor_ui,
                material,
                primitive,
                assignment.uv_transform,
            ) {
                return Some((
                    material.index,
                    (
                        assignment.mesh_id,
                        assignment.object_index,
                        assignment.primitive_index,
                    ),
                ));
            }
        }
    }

    None
}

fn uv_assignment_contains_pointer(
    pointer: egui::Pos2,
    atlas_rect: egui::Rect,
    editor_ui: &EditorUi,
    material: &AtlasMaterial,
    primitive: &crate::mesh::import::MeshPrimitive,
    uv_transform: crate::transform::UvTransform,
) -> bool {
    if primitive.uvs.is_empty() || primitive.indices.is_empty() {
        return false;
    }

    let atlas_position = screen_to_atlas(pointer, atlas_rect, editor_ui.atlas_zoom);

    let transform = &material.transform;

    let material_width = transform.rotated_width() as f32;
    let material_height = transform.rotated_height() as f32;

    if material_width <= 0.0 || material_height <= 0.0 {
        return false;
    }

    let local_x = (atlas_position.x - transform.x as f32) / material_width;

    let local_y = (atlas_position.y - transform.y as f32) / material_height;

    if !(0.0..=1.0).contains(&local_x) || !(0.0..=1.0).contains(&local_y) {
        return false;
    }

    let material_uv =
        inverse_material_rotation(Vec2::new(local_x, 1.0 - local_y), transform.rotation);

    let pivot = UvTransform::uv_center(&primitive.uvs);

    let uv = inverse_uv_transform(material_uv, pivot, uv_transform);

    for triangle in primitive.indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;

        let Some(&a) = primitive.uvs.get(a) else {
            continue;
        };

        let Some(&b) = primitive.uvs.get(b) else {
            continue;
        };

        let Some(&c) = primitive.uvs.get(c) else {
            continue;
        };

        if point_in_triangle(
            uv,
            Vec2::from_array(a),
            Vec2::from_array(b),
            Vec2::from_array(c),
        ) {
            return true;
        }
    }

    false
}

fn inverse_uv_transform(uv: Vec2, pivot: Vec2, transform: crate::transform::UvTransform) -> Vec2 {
    let mut result = uv - pivot - transform.translation;

    let inverse_rotation = bevy::math::Mat2::from_angle(-transform.rotation);

    result = inverse_rotation * result;

    if transform.scale.x.abs() > f32::EPSILON {
        result.x /= transform.scale.x;
    }

    if transform.scale.y.abs() > f32::EPSILON {
        result.y /= transform.scale.y;
    }

    result + pivot
}

fn inverse_material_rotation(uv: Vec2, rotation: crate::transform::Rotation) -> Vec2 {
    match rotation {
        crate::transform::Rotation::Deg0 => uv,

        crate::transform::Rotation::Deg90 => Vec2::new(uv.y, 1.0 - uv.x),

        crate::transform::Rotation::Deg180 => Vec2::new(1.0 - uv.x, 1.0 - uv.y),

        crate::transform::Rotation::Deg270 => Vec2::new(1.0 - uv.y, uv.x),
    }
}

fn point_in_triangle(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let ab = b - a;
    let bc = c - b;
    let ca = a - c;

    let ap = point - a;
    let bp = point - b;
    let cp = point - c;

    let cross1 = ab.perp_dot(ap);
    let cross2 = bc.perp_dot(bp);
    let cross3 = ca.perp_dot(cp);

    let epsilon = 0.00001;

    (cross1 >= -epsilon && cross2 >= -epsilon && cross3 >= -epsilon)
        || (cross1 <= epsilon && cross2 <= epsilon && cross3 <= epsilon)
}
