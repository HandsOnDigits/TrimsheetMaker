use bevy::{
    camera::RenderTarget,
    material::OpaqueRendererMethod,
    pbr::{ExtendedMaterial, StandardMaterial},
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
};

use super::{
    MeshPreview, MeshPreviewCamera, MeshPreviewExtension, MeshPreviewLightSource,
    MeshPreviewMaterial, MeshPreviewRenderTarget, MeshViewportLayout, MeshViewportSection,
    TransformMode,
};

use crate::rendering::DefaultTextures;

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

        selected_mesh_id: None,
        selected_primitive: None,
        selected_primitive_center: None,
        selected_primitive_normal: None,

        material,
        uv_revision: 0,

        transform_mode: TransformMode::None,
        uv_history_active: false,
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

pub fn update_mesh_viewport(
    layout: Res<MeshViewportLayout>,
    mut query: Query<&mut Node, With<MeshViewportSection>>,
) {
    for mut node in &mut query {
        node.left = Val::Px(layout.x);
        node.top = Val::Px(layout.y);
        node.width = Val::Px(layout.width);
        node.height = Val::Px(layout.height);
    }
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
