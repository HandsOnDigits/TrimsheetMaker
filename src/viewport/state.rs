use std::collections::HashMap;

use bevy::prelude::*;

use super::{MeshPreviewMaterial, MeshPrimitiveSelection};

#[derive(Component)]
pub struct MeshPreviewCamera;

#[derive(Resource, Default)]
pub struct MeshViewportInput {
    pub pointer_over_ui: bool,
    pub floating_window_dragging: bool,
}

#[derive(Component)]
pub struct MeshViewportSection;

#[derive(Resource)]
pub struct MeshPreview {
    pub camera: Entity,
    pub target: Vec3,
    pub distance: f32,
    pub target_distance: f32,
    pub yaw: f32,
    pub pitch: f32,

    pub selected_mesh_id: Option<u64>,
    pub selected_primitive: Option<MeshPrimitiveSelection>,

    pub material: Handle<MeshPreviewMaterial>,
    pub uv_revision: u64,
}

impl MeshPreview {
    pub fn clear_selection(&mut self) {
        self.selected_mesh_id = None;
        self.selected_primitive = None;
    }
}

#[derive(Resource, Default)]
pub struct MeshViewportLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Component)]
pub struct MeshPreviewLightSource;

#[derive(Resource, Default)]
pub struct MeshPreviewMaterials {
    pub by_atlas_material: HashMap<usize, Handle<MeshPreviewMaterial>>,
    pub default: Option<Handle<MeshPreviewMaterial>>,
}

#[derive(Resource)]
pub struct MeshPreviewRenderTarget {
    pub image: Handle<Image>,
    pub width: u32,
    pub height: u32,
}
