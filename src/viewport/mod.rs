mod atlas;
pub use atlas::{
    AtlasRefresh, ensure_atlas_build_requested, finish_atlas_build, sync_atlas_images,
};

mod camera;
mod deletion;
mod lighting;
mod materials;
mod picking;
mod setup;
mod state;

mod uv;
pub use uv::{MeshUvSyncState, sync_mesh_uvs};

mod visibility;

pub use camera::orbit_camera;

pub use lighting::update_mesh_light;

pub use materials::{
    MeshPreviewExtension, MeshPreviewMaterial, sync_atlas_to_mesh_preview_material,
    sync_mesh_preview_material_assignments,
};

pub use deletion::{despawn_all_meshes, remove_selected_mesh_primitive};

pub use picking::{MeshPrimitiveSelection, MeshViewportPicking, pick_mesh_primitive};

pub use setup::{
    resize_mesh_preview_render_target, setup_mesh_preview, spawn_mesh_viewport,
    update_mesh_viewport,
};

pub use state::{
    MeshPreview, MeshPreviewCamera, MeshPreviewLightSource, MeshPreviewMaterials,
    MeshPreviewRenderTarget, MeshViewportInput, MeshViewportLayout, MeshViewportSection,
    TransformMode,
};

pub use visibility::sync_mesh_visibility;

use bevy::{pbr::MaterialPlugin, prelude::*};
use bevy_egui::EguiPrimaryContextPass;

use crate::mesh::{MeshLight, MeshSceneState};

pub struct MeshPreviewPlugin;

impl Plugin for MeshPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<MeshPreviewMaterial>::default())
            .init_resource::<MeshViewportLayout>()
            .init_resource::<MeshLight>()
            .init_resource::<MeshViewportInput>()
            .init_resource::<MeshViewportPicking>()
            .init_resource::<MeshSceneState>()
            .init_resource::<AtlasRefresh>()
            .init_resource::<MeshPreviewMaterials>()
            .init_resource::<MeshUvSyncState>()
            .add_systems(Startup, (setup_mesh_preview, spawn_mesh_viewport).chain())
            .add_systems(
                EguiPrimaryContextPass,
                (
                    update_mesh_viewport,
                    orbit_camera.after(crate::ui::panels::editor_ui),
                    pick_mesh_primitive.after(orbit_camera),
                    remove_selected_mesh_primitive.after(pick_mesh_primitive),
                ),
            )
            .add_systems(
                Update,
                (
                    resize_mesh_preview_render_target,
                    sync_mesh_visibility,
                    ensure_atlas_build_requested,
                    sync_atlas_images,
                    finish_atlas_build,
                    sync_atlas_to_mesh_preview_material.after(finish_atlas_build),
                    sync_mesh_preview_material_assignments
                        .after(sync_atlas_to_mesh_preview_material),
                    sync_mesh_uvs.after(sync_mesh_preview_material_assignments),
                    update_mesh_light,
                ),
            );
    }
}

pub fn clear_all_meshes(commands: &mut Commands, mesh_scene_state: &mut MeshSceneState) {
    for mesh in &mut mesh_scene_state.meshes {
        if let Some(root) = mesh.root.take() {
            commands.entity(root).despawn();
        }
    }

    mesh_scene_state.meshes.clear();
}
