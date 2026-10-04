use bevy::{
    camera::RenderTarget,
    material::OpaqueRendererMethod,
    pbr::{ExtendedMaterial, StandardMaterial},
    prelude::*,
    render::render_resource::TextureFormat,
};

use super::{
    MeshPreview, MeshPreviewCamera, MeshPreviewExtension, MeshPreviewLightSource,
    MeshPreviewMaterial, MeshPreviewRenderTarget, MeshUvSyncState, MeshViewportSection,
};

use crate::{
    data::Atlas,
    mesh::{
        ImportedMeshPart, ImportedMeshRoot, MeshSceneState, create_imported_bevy_mesh,
        unwrap::{MeshUvMode, create_unwrapped_bevy_mesh},
    },
    rendering::DefaultTextures,
};

pub fn setup_mesh_preview(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<MeshPreviewMaterial>>,
) {
    let width = 1024;
    let height = 1024;

    let image = Image::new_target_texture(width, height, TextureFormat::Rgba8UnormSrgb, None);

    let image_handle = images.add(image);

    commands.insert_resource(MeshPreviewRenderTarget {
        image: image_handle.clone(),
        width,
        height,
    });

    let camera = commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: -1,
                ..default()
            },
            RenderTarget::Image(image_handle.into()),
            Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            MeshPreviewCamera,
        ))
        .id();

    let default_textures = DefaultTextures::create(&mut images);

    let material = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            opaque_render_method: OpaqueRendererMethod::Forward,
            ..default()
        },

        extension: MeshPreviewExtension {
            base_color_texture: Some(default_textures.checkerboard.clone()),
            normal_texture: Some(default_textures.flat_normal.clone()),
            ambient_occlusion_texture: Some(default_textures.white.clone()),
            roughness_texture: Some(default_textures.white.clone()),
            metallic_texture: Some(default_textures.black.clone()),
        },
    });

    commands.insert_resource(default_textures);

    commands.insert_resource(MeshPreview {
        camera,
        target: Vec3::ZERO,
        distance: 4.0,
        target_distance: 4.0,
        yaw: 0.0,
        pitch: 0.3,

        uv_mode: MeshUvMode::Imported,
        unwrapped_scene: None,
        selected_mesh_id: None,
        unwrapped_mesh_id: None,
        selected_primitive: None,

        material,
        uv_revision: 0,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(2.0, 4.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        MeshPreviewLightSource,
    ));
}

pub fn spawn_mesh_viewport(mut commands: Commands, mesh_preview: Res<MeshPreview>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(0.0),
            height: Val::Px(0.0),
            ..default()
        },
        MeshViewportSection,
        children![(
            ViewportNode::new(mesh_preview.camera),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        )],
    ));
}

pub fn despawn_all_meshes(commands: &mut Commands, scene_state: &mut MeshSceneState) {
    for mesh in scene_state.meshes.drain(..) {
        if let Some(root) = mesh.root {
            commands.entity(root).despawn();
        }
    }

    scene_state.next_id = 0;
}

pub fn sync_mesh_visibility(
    scene_state: Res<MeshSceneState>,
    mut root_query: Query<
        (&ImportedMeshRoot, &mut Visibility),
        (With<ImportedMeshRoot>, Without<ImportedMeshPart>),
    >,
    mut part_query: Query<
        (&ImportedMeshPart, &mut Visibility),
        (With<ImportedMeshPart>, Without<ImportedMeshRoot>),
    >,
) {
    for (root, mut visibility) in &mut root_query {
        let Some(mesh) = scene_state.get(root.id) else {
            continue;
        };

        *visibility = if mesh.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (part, mut visibility) in &mut part_query {
        let Some(mesh) = scene_state.get(part.mesh_id) else {
            continue;
        };

        let Some(part_entry) = mesh
            .parts
            .iter()
            .find(|entry| entry.index == part.part_index)
        else {
            continue;
        };

        *visibility = if mesh.visible && part_entry.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn sync_mesh_uv_mode(
    mesh_preview: Res<MeshPreview>,
    atlas: Res<Atlas>,
    mesh_scene_state: Res<MeshSceneState>,
    mut sync_state: ResMut<MeshUvSyncState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&ImportedMeshPart, &mut Mesh3d)>,
) {
    if sync_state.mesh_revision == mesh_preview.uv_revision
        && sync_state.atlas_revision == atlas.uv_revision
        && sync_state.uv_mode == mesh_preview.uv_mode
    {
        return;
    }

    sync_state.mesh_revision = mesh_preview.uv_revision;
    sync_state.atlas_revision = atlas.uv_revision;
    sync_state.uv_mode = mesh_preview.uv_mode;

    for (part, mut mesh3d) in &mut query {
        let Some(mesh_entry) = mesh_scene_state.get(part.mesh_id) else {
            continue;
        };

        let Some(primitive_index) = part.primitive_index else {
            continue;
        };

        let Some(part_entry) = mesh_entry
            .parts
            .iter()
            .find(|entry| entry.index == part.part_index)
        else {
            continue;
        };

        let object_index = part_entry.object_index;

        let Some(object) = mesh_entry.scene.objects.get(object_index) else {
            continue;
        };

        let Some(primitive) = object.primitives.get(primitive_index) else {
            continue;
        };

        let material = atlas.materials.iter().find(|material| {
            material.has_mesh_assignment(part.mesh_id, object_index, primitive_index)
        });

        match mesh_preview.uv_mode {
            MeshUvMode::Imported => {
                let uvs = super::atlas::material_uvs(&primitive.uvs, material, &atlas);

                mesh3d.0 = create_imported_bevy_mesh(&mut meshes, primitive, uvs);
            }

            MeshUvMode::Unwrapped => {
                if mesh_preview.unwrapped_mesh_id != Some(part.mesh_id) {
                    continue;
                }

                let Some(unwrapped_scene) = mesh_preview.unwrapped_scene.as_ref() else {
                    continue;
                };

                let Some(unwrapped_object) = unwrapped_scene.objects.get(object_index) else {
                    continue;
                };

                let Some(unwrapped) = unwrapped_object.primitives.get(primitive_index) else {
                    continue;
                };

                let uvs = super::atlas::material_uvs(&unwrapped.uvs, material, &atlas);

                mesh3d.0 = create_unwrapped_bevy_mesh(&mut meshes, unwrapped, uvs);
            }
        }
    }
}
