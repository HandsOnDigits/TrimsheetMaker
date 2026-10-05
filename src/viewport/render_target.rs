use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;

use super::MeshViewportLayout;

#[derive(Resource)]
pub struct MeshPreviewRenderTarget {
    pub image: Handle<Image>,
    pub width: u32,
    pub height: u32,
}

pub fn resize_mesh_preview_render_target(
    layout: Res<MeshViewportLayout>,
    mut target: ResMut<MeshPreviewRenderTarget>,
    mut images: ResMut<Assets<Image>>,
) {
    let width = layout.width.max(1.0).round() as u32;
    let height = layout.height.max(1.0).round() as u32;

    if target.width == width && target.height == height {
        return;
    }

    let Some(mut image) = images.get_mut(&target.image) else {
        return;
    };

    image.resize(Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    });

    target.width = width;
    target.height = height;
}
