use bevy::{
    asset::RenderAssetUsages,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::render_resource::PrimitiveTopology,
};

use super::{
    MeshLight, MeshLightPreset, MeshPreview, MeshPreviewCamera, MeshPreviewLight, MeshSceneState,
    MeshViewportInput, MeshViewportLayout,
};

use crate::ui::EditorUi;

#[derive(Component)]
pub struct ImportedMesh;

#[derive(Component)]
pub struct MeshViewportSection;

pub fn spawn_mesh_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    scene: &super::import::MeshScene,
) -> Entity {
    let root = commands
        .spawn((
            Name::new("Imported Mesh"),
            Transform::default(),
            ImportedMesh,
        ))
        .id();

    for object in &scene.objects {
        let object_entity = commands
            .spawn((
                Name::new(object.name.clone()),
                Transform::from_translation(Vec3::from(object.transform.translation))
                    .with_rotation(Quat::from_xyzw(
                        object.transform.rotation[0],
                        object.transform.rotation[1],
                        object.transform.rotation[2],
                        object.transform.rotation[3],
                    ))
                    .with_scale(Vec3::from(object.transform.scale)),
                ImportedMesh,
            ))
            .id();

        commands.entity(root).add_child(object_entity);

        for (primitive_index, primitive) in object.primitives.iter().enumerate() {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, primitive.positions.clone());

            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, primitive.normals.clone());

            mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, primitive.tangents.clone());

            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, primitive.uvs.clone());

            mesh.insert_indices(bevy::mesh::Indices::U32(primitive.indices.clone()));

            let mesh_handle = meshes.add(mesh);

            let material_handle = materials.add(StandardMaterial {
                base_color: Color::srgb(0.7, 0.7, 0.7),
                perceptual_roughness: 0.7,
                ..default()
            });

            commands.entity(object_entity).with_children(|parent| {
                parent.spawn((
                    Name::new(format!("{} Primitive {}", object.name, primitive_index)),
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    ImportedMesh,
                ));
            });
        }
    }

    root
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

pub fn despawn_imported_mesh(commands: &mut Commands, scene_state: &mut MeshSceneState) {
    if let Some(root) = scene_state.root.take() {
        commands.entity(root).despawn();
    }
}

pub fn update_mesh_light(
    mesh_light: Res<MeshLight>,
    mesh_preview: Res<MeshPreview>,
    mut lights: Query<(&mut DirectionalLight, &mut Transform), With<MeshPreviewLight>>,
) {
    if !mesh_light.is_changed() {
        return;
    }

    let Ok((mut light, mut transform)) = lights.single_mut() else {
        return;
    };

    match mesh_light.preset {
        MeshLightPreset::Studio => {
            light.illuminance = 10_000.0;
            light.color = Color::srgb(1.0, 1.0, 1.0);
        }

        MeshLightPreset::Neutral => {
            light.illuminance = 7_000.0;
            light.color = Color::srgb(1.0, 1.0, 1.0);
        }

        MeshLightPreset::Warm => {
            light.illuminance = 8_000.0;
            light.color = Color::srgb(1.0, 0.78, 0.58);
        }

        MeshLightPreset::Cool => {
            light.illuminance = 8_000.0;
            light.color = Color::srgb(0.65, 0.8, 1.0);
        }

        MeshLightPreset::Flat => {
            light.illuminance = 3_000.0;
            light.color = Color::srgb(1.0, 1.0, 1.0);
        }

        MeshLightPreset::None => {
            light.illuminance = 0.0;
        }
    }

    *transform =
        Transform::from_translation(mesh_light.position).looking_at(mesh_preview.target, Vec3::Y);
}

pub fn orbit_camera(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mesh_viewport: Res<MeshViewportLayout>,
    mesh_input: Res<MeshViewportInput>,
    editor_ui: Res<EditorUi>,
    time: Res<Time>,
    mut mesh_preview: ResMut<MeshPreview>,
    mut mesh_light: ResMut<MeshLight>,
    cameras: Query<(&Camera, &GlobalTransform), With<MeshPreviewCamera>>,
    mut camera_transforms: Query<&mut Transform, With<MeshPreviewCamera>>,
) {
    let Ok(window) = windows.single() else {
        mouse_motion.clear();
        mouse_wheel.clear();
        return;
    };

    let cursor_position = window.cursor_position();

    let cursor_inside = cursor_position.is_some_and(|position| mesh_viewport.contains(position));

    // -------------------------------------------------------------
    // Light target
    // -------------------------------------------------------------

    if mesh_light.follow_mouse && cursor_inside && !mesh_input.pointer_over_ui {
        if let Some(cursor) = cursor_position {
            let Ok((camera, camera_global)) = cameras.single() else {
                return;
            };

            if let Ok(ray) = camera.viewport_to_world(camera_global, cursor) {
                /*
                 * Create a plane through the mesh target.
                 *
                 * The plane faces the camera, so the mouse cursor
                 * always corresponds to a point on the mesh's
                 * screen-facing plane.
                 */
                let plane_origin = mesh_preview.target;
                let plane_normal = *camera_global.forward();

                let denominator = ray.direction.dot(plane_normal);

                if denominator.abs() > 0.0001 {
                    let distance = (plane_origin - ray.origin).dot(plane_normal) / denominator;

                    if distance >= 0.0 {
                        let target = ray.origin + ray.direction * distance;

                        /*
                         * Move the light to the position represented
                         * by the cursor while keeping it at a fixed
                         * distance from the mesh.
                         */
                        let offset = target - mesh_preview.target;

                        if offset.length_squared() > 0.0001 {
                            mesh_light.position = offset.normalize() * 5.0;
                        }
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------
    // Camera orbit
    // -------------------------------------------------------------

    if mesh_input.pointer_over_ui {
        mesh_preview.orbiting = false;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) && cursor_inside && !mesh_input.pointer_over_ui
    {
        mesh_preview.orbiting = true;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        mesh_preview.orbiting = false;
    }

    if mesh_preview.orbiting {
        let mut delta = Vec2::ZERO;

        for event in mouse_motion.read() {
            delta += event.delta;
        }

        let sensitivity = 0.01;

        let yaw = Quat::from_rotation_y(-delta.x * sensitivity);
        let pitch = Quat::from_rotation_x(-delta.y * sensitivity);

        mesh_preview.rotation = yaw * mesh_preview.rotation;
        mesh_preview.rotation = mesh_preview.rotation * pitch;
    } else {
        mouse_motion.clear();
    }

    // -------------------------------------------------------------
    // Zoom
    // -------------------------------------------------------------

    let ctrl_pressed =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if cursor_inside {
        let mut scroll = 0.0;

        for event in mouse_wheel.read() {
            scroll += event.y;
        }

        if scroll != 0.0 && ctrl_pressed && !mesh_input.pointer_over_ui {
            let zoom_speed = editor_ui.editor_settings.mouse_scroll_speed;

            mesh_preview.target_distance *= (-scroll * zoom_speed * 0.01).exp();

            mesh_preview.target_distance = mesh_preview.target_distance.clamp(0.25, 100.0);
        }
    } else {
        mouse_wheel.clear();
    }

    // -------------------------------------------------------------
    // Camera smoothing
    // -------------------------------------------------------------

    let smoothing = 12.0;

    let interpolation = 1.0 - (-smoothing * time.delta_secs()).exp();

    mesh_preview.distance = mesh_preview
        .distance
        .lerp(mesh_preview.target_distance, interpolation);

    let offset = mesh_preview.rotation * Vec3::new(0.0, 0.0, mesh_preview.distance);

    for mut transform in &mut camera_transforms {
        transform.translation = mesh_preview.target + offset;

        transform.look_at(mesh_preview.target, Vec3::Y);
    }
}
