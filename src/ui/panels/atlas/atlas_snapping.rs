use bevy_egui::egui;

use crate::data::Atlas;

use super::atlas_interaction::material_rect_in_atlas;

pub fn move_material_with_snapping(
    atlas: &mut Atlas,
    index: usize,
    desired_x: f32,
    desired_y: f32,
    snap_distance: f32,
) {
    let Some(dragged_index) = atlas.materials.iter().position(|m| m.index == index) else {
        return;
    };

    let padding = atlas.padding as f32;

    let dragged = &atlas.materials[dragged_index];

    let current_rect = material_rect_in_atlas(dragged);

    let offset_x = current_rect.left() - dragged.transform.x as f32;

    let offset_y = current_rect.top() - dragged.transform.y as f32;

    let desired_rect = egui::Rect::from_min_size(
        egui::pos2(desired_x + offset_x, desired_y + offset_y),
        current_rect.size(),
    );

    let mut final_rect = desired_rect;

    snap_x(
        atlas,
        dragged_index,
        &desired_rect,
        padding,
        snap_distance,
        &mut final_rect,
    );

    snap_y(
        atlas,
        dragged_index,
        &desired_rect,
        padding,
        snap_distance,
        &mut final_rect,
    );

    let final_x = final_rect.left() - offset_x;

    let final_y = final_rect.top() - offset_y;

    let material = &mut atlas.materials[dragged_index];

    material.transform.x = final_x.round() as i32;

    material.transform.y = final_y.round() as i32;
}

fn snap_x(
    atlas: &Atlas,
    dragged_index: usize,
    rect: &egui::Rect,
    padding: f32,
    snap_distance: f32,
    result: &mut egui::Rect,
) {
    let mut best: Option<SnapCandidate> = None;

    let atlas_width = atlas.width as f32;

    // -------------------------------------------------
    // Atlas edges
    // -------------------------------------------------

    add_candidate(&mut best, rect.left(), padding, snap_distance);

    add_candidate(
        &mut best,
        rect.right(),
        atlas_width - padding,
        snap_distance,
    );

    // -------------------------------------------------
    // Other materials
    // -------------------------------------------------

    for (material_index, material) in atlas.materials.iter().enumerate() {
        if material_index == dragged_index {
            continue;
        }

        let other = material_rect_in_atlas(material);

        // X snapping only makes sense when the rectangles
        // overlap vertically.
        if !ranges_overlap(rect.top(), rect.bottom(), other.top(), other.bottom()) {
            continue;
        }

        // Our left edge -> other's right edge + margin.
        add_candidate(
            &mut best,
            rect.left(),
            other.right() + padding,
            snap_distance,
        );

        // Our right edge -> other's left edge - margin.
        add_candidate(
            &mut best,
            rect.right(),
            other.left() - padding,
            snap_distance,
        );

        // -------------------------------------------------
        // Edge alignment
        // -------------------------------------------------

        // Left -> left.
        add_candidate(&mut best, rect.left(), other.left(), snap_distance);

        // Right -> right.
        add_candidate(&mut best, rect.right(), other.right(), snap_distance);

        // Center -> center.
        add_candidate(&mut best, rect.center().x, other.center().x, snap_distance);
    }

    if let Some(candidate) = best {
        let delta = candidate.target - candidate.current;

        *result = result.translate(egui::vec2(delta, 0.0));
    }
}

fn snap_y(
    atlas: &Atlas,
    dragged_index: usize,
    rect: &egui::Rect,
    margin: f32,
    snap_distance: f32,
    result: &mut egui::Rect,
) {
    let mut best: Option<SnapCandidate> = None;

    let atlas_height = atlas.height as f32;

    // -------------------------------------------------
    // Atlas edges
    // -------------------------------------------------

    add_candidate(&mut best, rect.top(), margin, snap_distance);

    add_candidate(
        &mut best,
        rect.bottom(),
        atlas_height - margin,
        snap_distance,
    );

    // -------------------------------------------------
    // Other materials
    // -------------------------------------------------

    for (material_index, material) in atlas.materials.iter().enumerate() {
        if material_index == dragged_index {
            continue;
        }

        let other = material_rect_in_atlas(material);

        // Y snapping only makes sense when the rectangles
        // overlap horizontally.
        if !ranges_overlap(rect.left(), rect.right(), other.left(), other.right()) {
            continue;
        }

        // Our top -> other's bottom + margin.
        add_candidate(
            &mut best,
            rect.top(),
            other.bottom() + margin,
            snap_distance,
        );

        // Our bottom -> other's top - margin.
        add_candidate(
            &mut best,
            rect.bottom(),
            other.top() - margin,
            snap_distance,
        );

        // -------------------------------------------------
        // Edge alignment
        // -------------------------------------------------

        // Top -> top.
        add_candidate(&mut best, rect.top(), other.top(), snap_distance);

        // Bottom -> bottom.
        add_candidate(&mut best, rect.bottom(), other.bottom(), snap_distance);

        // Center -> center.
        add_candidate(&mut best, rect.center().y, other.center().y, snap_distance);
    }

    if let Some(candidate) = best {
        let delta = candidate.target - candidate.current;

        *result = result.translate(egui::vec2(0.0, delta));
    }
}

#[derive(Clone, Copy)]
struct SnapCandidate {
    current: f32,
    target: f32,
    distance: f32,
}

fn add_candidate(best: &mut Option<SnapCandidate>, current: f32, target: f32, snap_distance: f32) {
    let distance = (current - target).abs();

    if distance > snap_distance {
        return;
    }

    let candidate = SnapCandidate {
        current,
        target,
        distance,
    };

    let replace = match best {
        None => true,
        Some(previous) => candidate.distance < previous.distance,
    };

    if replace {
        *best = Some(candidate);
    }
}

fn ranges_overlap(a_min: f32, a_max: f32, b_min: f32, b_max: f32) -> bool {
    a_min < b_max && a_max > b_min
}
