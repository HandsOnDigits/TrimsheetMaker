use std::collections::HashMap;

use bevy::prelude::*;

use crate::mesh::unwrap::{MeshUvMode, UnwrappedMeshScene};

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

    pub uv_mode: MeshUvMode,
    pub unwrapped_scene: Option<UnwrappedMeshScene>,
    pub selected_mesh_id: Option<u64>,
    pub unwrapped_mesh_id: Option<u64>,
    pub selected_primitive: Option<MeshPrimitiveSelection>,

    pub material: Handle<MeshPreviewMaterial>,
    pub uv_revision: u64,
}

impl MeshPreview {
    pub fn clear_selection(&mut self) {
        self.selected_mesh_id = None;
        self.selected_primitive = None;
        self.unwrapped_mesh_id = None;
        self.unwrapped_scene = None;
    }
}

#[derive(Resource, Default)]
pub struct MeshViewportLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
pub struct MeshUvSyncState {
    pub mesh_revision: u64,
    pub atlas_revision: u64,
    pub uv_mode: MeshUvMode,
}

impl Default for MeshUvSyncState {
    fn default() -> Self {
        Self {
            mesh_revision: u64::MAX,
            atlas_revision: u64::MAX,
            uv_mode: MeshUvMode::Imported,
        }
    }
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
