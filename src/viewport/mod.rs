mod atlas;
pub use atlas::{
    AtlasRefresh, ensure_atlas_build_requested, finish_atlas_build, sync_atlas_images,
};

mod camera;
mod deletion;
mod lighting;
mod materials;

mod picking;
pub use picking::MeshViewportContextMenu;

mod setup;
mod state;

mod visibility;

pub use camera::orbit_camera;

pub use lighting::update_mesh_light;

pub use materials::{
    MeshPreviewExtension, MeshPreviewMaterial, sync_atlas_to_mesh_preview_material,
};

pub use deletion::despawn_all_meshes;

pub use picking::{
    MeshPrimitiveSelection, MeshViewportPicking, pick_mesh_primitive, sync_mesh_primitive_wireframe,
};

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

use bevy::{
    pbr::{MaterialPlugin, wireframe::WireframePlugin},
    prelude::*,
};
use bevy_egui::EguiPrimaryContextPass;

use crate::{
    data::Atlas,
    mesh::{
        ImportedMeshPart, MeshLight, MeshSceneState,
        uv::{MeshUvSyncState, sync_mesh_uvs},
    },
};

pub struct MeshPreviewPlugin;

impl Plugin for MeshPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<MeshPreviewMaterial>::default(),
            WireframePlugin::default(),
        ))
        .init_resource::<MeshViewportLayout>()
        .init_resource::<MeshLight>()
        .init_resource::<MeshViewportInput>()
        .init_resource::<MeshViewportPicking>()
        .init_resource::<MeshViewportContextMenu>()
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
                sync_mesh_primitive_wireframe.after(pick_mesh_primitive),
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
                sync_mesh_preview_material_assignments.after(sync_atlas_to_mesh_preview_material),
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

pub fn sync_mesh_preview_material_assignments(
    atlas: Res<Atlas>,
    preview_materials: Res<MeshPreviewMaterials>,
    mesh_scene_state: Res<MeshSceneState>,
    mut commands: Commands,
    parts: Query<(Entity, &ImportedMeshPart)>,
) {
    for (entity, part) in &parts {
        let Some(mesh_entry) = mesh_scene_state.get(part.mesh_id) else {
            if let Some(handle) = &preview_materials.default {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(handle.clone()));
            }

            continue;
        };

        let Some(mesh_part) = mesh_entry
            .parts
            .iter()
            .find(|entry| entry.index == part.part_index)
        else {
            if let Some(handle) = &preview_materials.default {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(handle.clone()));
            }

            continue;
        };

        let object_index = mesh_part.object_index;
        let primitive_index = mesh_part.primitive_index;

        let Some(atlas_material) = atlas.materials.iter().find(|material| {
            material.has_mesh_assignment(part.mesh_id, object_index, primitive_index)
        }) else {
            if let Some(handle) = &preview_materials.default {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(handle.clone()));
            }

            continue;
        };

        let Some(handle) = preview_materials
            .by_atlas_material
            .get(&atlas_material.index)
        else {
            if let Some(handle) = &preview_materials.default {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(handle.clone()));
            }

            continue;
        };

        commands
            .entity(entity)
            .insert(MeshMaterial3d(handle.clone()));
    }
}
