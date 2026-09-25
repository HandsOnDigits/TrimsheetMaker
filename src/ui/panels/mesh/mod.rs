pub mod mesh_toolbar;
pub mod mesh_view;

use bevy_egui::egui;

use crate::mesh::{MeshLight, MeshPreview, MeshViewportInput};

pub fn mesh_panel(
    ctx: &egui::Context,
    rect: egui::Rect,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
    mesh_input: &mut MeshViewportInput,
) {
    mesh_view::mesh_view(ctx, rect, mesh_preview, mesh_light, mesh_input);
}
