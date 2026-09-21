use bevy_egui::egui;

use crate::data::Atlas;

use super::atlas_interaction::material_rect_in_atlas;

pub fn move_material_with_snapping(
    atlas: &mut Atlas,
    index: usize,
    desired_x: f32,
    desired_y: f32,
    margin: f32,
    snap_distance: f32,
) {
    let Some(dragged_index) = atlas.materials.iter().position(|m| m.index == index) else {
        return;
    };

    let dragged = &atlas.materials[dragged_index];

    let current_rect = material_rect_in_atlas(dragged);

    // Offset between Transform position and the visible rectangle.
    let offset_x = current_rect.left() - dragged.transform.x as f32;

    let offset_y = current_rect.top() - dragged.transform.y as f32;

    // Position the rectangle where the cursor wants it.
    let rect = egui::Rect::from_min_size(
        egui::pos2(desired_x + offset_x, desired_y + offset_y),
        current_rect.size(),
    );

    let mut snapped_left = rect.left();
    let mut snapped_top = rect.top();

    let mut best_x = snap_distance;
    let mut best_y = snap_distance;

    snap_to_atlas_edges(
        atlas,
        &rect,
        margin,
        &mut snapped_left,
        &mut best_x,
        &mut snapped_top,
        &mut best_y,
    );

    snap_to_materials(
        atlas,
        dragged_index,
        &rect,
        margin,
        &mut snapped_left,
        &mut best_x,
        &mut snapped_top,
        &mut best_y,
    );

    // Convert the snapped rectangle position back to Transform coordinates.
    let final_x = snapped_left - offset_x;
    let final_y = snapped_top - offset_y;

    let material = &mut atlas.materials[dragged_index];

    material.transform.x = final_x.round() as i32;
    material.transform.y = final_y.round() as i32;
}

fn snap_to_atlas_edges(
    atlas: &Atlas,
    rect: &egui::Rect,
    margin: f32,
    snapped_x: &mut f32,
    best_x: &mut f32,
    snapped_y: &mut f32,
    best_y: &mut f32,
) {
    let atlas_width = atlas.width as f32;
    let atlas_height = atlas.height as f32;

    // Left edge.
    try_snap(rect.left(), margin, best_x, snapped_x);

    // Right edge.
    try_snap(rect.right(), atlas_width - margin, best_x, snapped_x);

    // Top edge.
    try_snap(rect.top(), margin, best_y, snapped_y);

    // Bottom edge.
    try_snap(rect.bottom(), atlas_height - margin, best_y, snapped_y);
}

fn snap_to_materials(
    atlas: &Atlas,
    dragged_index: usize,
    rect: &egui::Rect,
    margin: f32,
    snapped_x: &mut f32,
    best_x: &mut f32,
    snapped_y: &mut f32,
    best_y: &mut f32,
) {
    for (index, material) in atlas.materials.iter().enumerate() {
        if index == dragged_index {
            continue;
        }

        let other = material_rect_in_atlas(material);

        // -------------------------------------------------
        // Horizontal snapping
        // -------------------------------------------------

        // Our left edge -> other right edge + margin.
        try_snap(rect.left(), other.right() + margin, best_x, snapped_x);

        // Our right edge -> other left edge - margin.
        try_snap(rect.right(), other.left() - margin, best_x, snapped_x);

        // -------------------------------------------------
        // Vertical snapping
        // -------------------------------------------------

        // Our top edge -> other bottom edge + margin.
        try_snap(rect.top(), other.bottom() + margin, best_y, snapped_y);

        // Our bottom edge -> other top edge - margin.
        try_snap(rect.bottom(), other.top() - margin, best_y, snapped_y);
    }
}

fn try_snap(current: f32, target: f32, best_distance: &mut f32, snapped: &mut f32) {
    let distance = (current - target).abs();

    if distance <= *best_distance {
        *best_distance = distance;
        *snapped = target;
    }
}
