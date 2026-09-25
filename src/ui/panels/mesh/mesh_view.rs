use bevy_egui::egui;

use crate::mesh::{MeshLight, MeshPreview};

use super::mesh_toolbar::mesh_toolbar;

pub fn mesh_view(
    ctx: &egui::Context,
    rect: egui::Rect,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
) {
    mesh_toolbar(ctx, rect, mesh_preview, mesh_light);
}
