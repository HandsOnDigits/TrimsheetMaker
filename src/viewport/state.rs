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

#[derive(Clone, Copy)]
pub struct MeshPreviewView {
    pub target: Vec3,
    pub distance: f32,
    pub target_distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformMode {
    #[default]
    None,
    Uv,
}

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
    pub selected_primitive_center: Option<Vec3>,
    pub selected_primitive_normal: Option<Vec3>,

    pub material: Handle<MeshPreviewMaterial>,
    pub uv_revision: u64,

    pub transform_mode: TransformMode,
    pub uv_history_active: bool,
}

impl MeshPreview {
    pub fn clear_selection(&mut self) {
        self.selected_mesh_id = None;
        self.selected_primitive = None;
        self.selected_primitive_center = None;
        self.selected_primitive_normal = None;
    }

    pub fn get_view(&self) -> MeshPreviewView {
        MeshPreviewView {
            target: self.target,
            distance: self.distance,
            target_distance: self.target_distance,
            yaw: self.yaw,
            pitch: self.pitch,
        }
    }

    pub fn set_view(&mut self, view: MeshPreviewView) {
        self.target = view.target;
        self.distance = view.distance;
        self.target_distance = view.target_distance;
        self.yaw = view.yaw;
        self.pitch = view.pitch;
    }

    pub fn reset_view(&mut self) {
        self.distance = 4.0;
        self.target_distance = 4.0;

        if let (Some(center), Some(normal)) = (
            self.selected_primitive_center,
            self.selected_primitive_normal,
        ) {
            self.target = center;

            let direction = normal.normalize_or_zero();
            let horizontal_length = direction.x.hypot(direction.z);

            if horizontal_length > 1e-5 {
                self.yaw = direction.x.atan2(direction.z);
            }

            self.pitch = (-direction.y).atan2(horizontal_length).clamp(-1.5, 1.5);
        } else {
            self.target = Vec3::ZERO;
            self.yaw = 0.0;
            self.pitch = 0.3;
        }
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
