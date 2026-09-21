use crate::data::{Atlas, AtlasMaterial};

use bevy_egui::egui;

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

    let dragged_rect = material_rect_in_atlas(dragged);

    let desired_rect = egui::Rect::from_min_size(
        egui::pos2(
            desired_x + dragged_rect.min.x - dragged.transform.x as f32,
            desired_y + dragged_rect.min.y - dragged.transform.y as f32,
        ),
        dragged_rect.size(),
    );

    let mut snapped_x = desired_rect.min.x;

    let mut snapped_y = desired_rect.min.y;

    let mut best_x = snap_distance + f32::EPSILON;

    let mut best_y = snap_distance + f32::EPSILON;

    snap_to_atlas_edges(
        atlas,
        &desired_rect,
        margin,
        &mut snapped_x,
        &mut best_x,
        &mut snapped_y,
        &mut best_y,
    );

    snap_to_materials(
        atlas,
        dragged_index,
        &desired_rect,
        margin,
        &mut snapped_x,
        &mut best_x,
        &mut snapped_y,
        &mut best_y,
    );

    let offset_x = dragged_rect.min.x - dragged.transform.x as f32;

    let offset_y = dragged_rect.min.y - dragged.transform.y as f32;

    let final_x = snapped_x - offset_x;

    let final_y = snapped_y - offset_y;

    let material = &mut atlas.materials[dragged_index];

    material.transform.x = final_x.round().max(0.0) as u32;

    material.transform.y = final_y.round().max(0.0) as u32;
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
    try_snap(rect.left(), margin, best_x, snapped_x);

    try_snap(
        rect.left(),
        atlas.width as f32 - margin - rect.width(),
        best_x,
        snapped_x,
    );

    try_snap(rect.top(), margin, best_y, snapped_y);

    try_snap(
        rect.top(),
        atlas.height as f32 - margin - rect.height(),
        best_y,
        snapped_y,
    );
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

        // Left of another material.
        try_snap(rect.left(), other.right() + margin, best_x, snapped_x);

        // Right of another material.
        try_snap(
            rect.left(),
            other.left() - margin - rect.width(),
            best_x,
            snapped_x,
        );

        // Above another material.
        try_snap(rect.top(), other.bottom() + margin, best_y, snapped_y);

        // Below another material.
        try_snap(
            rect.top(),
            other.top() - margin - rect.height(),
            best_y,
            snapped_y,
        );
    }
}

fn try_snap(current: f32, target: f32, best_distance: &mut f32, snapped: &mut f32) {
    let distance = (current - target).abs();

    if distance < *best_distance {
        *best_distance = distance;
        *snapped = target;
    }
}

fn material_rect_in_atlas(material: &AtlasMaterial) -> egui::Rect {
    let transform = &material.transform;

    let center_x = transform.x as f32 + transform.width as f32 * 0.5;

    let center_y = transform.y as f32 + transform.height as f32 * 0.5;

    let width = transform.rotated_width() as f32;

    let height = transform.rotated_height() as f32;

    egui::Rect::from_center_size(egui::pos2(center_x, center_y), egui::vec2(width, height))
}
