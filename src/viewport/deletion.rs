use bevy::prelude::*;

use crate::mesh::MeshSceneState;

pub fn despawn_all_meshes(commands: &mut Commands, scene_state: &mut MeshSceneState) {
    for mesh in &mut scene_state.meshes {
        if let Some(root) = mesh.root.take() {
            commands.entity(root).despawn();
        }

        mesh.parts.clear();
        mesh.bounds_center = Vec3::ZERO;
        mesh.bounds_radius = 0.0;
    }
}
