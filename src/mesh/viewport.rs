use bevy::{
    asset::RenderAssetUsages,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::render_resource::PrimitiveTopology,
};

use super::{
    ImportedMeshRoot, MeshLight, MeshLightPreset, MeshPartEntry, MeshPreview, MeshPreviewCamera,
    MeshPreviewLight, MeshSceneState, MeshViewportInput, MeshViewportLayout,
};

use crate::ui::EditorUi;

#[derive(Component)]
pub struct ImportedMesh;

#[derive(Component)]
pub struct ImportedMeshPart {
    pub mesh_id: u64,
    pub part_index: usize,
}

#[derive(Component)]
pub struct MeshViewportSection;

pub fn spawn_mesh_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    scene: &super::import::MeshScene,
    mesh_id: u64,
) -> (Entity, Vec<MeshPartEntry>, Vec3, f32) {
    let mut bounds_min = Vec3::splat(f32::INFINITY);
    let mut bounds_max = Vec3::splat(f32::NEG_INFINITY);

    for object in &scene.objects {
        let translation = Vec3::from(object.transform.translation);

        let rotation = Quat::from_xyzw(
            object.transform.rotation[0],
            object.transform.rotation[1],
            object.transform.rotation[2],
            object.transform.rotation[3],
        );

        let scale = Vec3::from(object.transform.scale);

        let transform = Transform {
            translation,
            rotation,
            scale,
        };

        for primitive in &object.primitives {
            for position in &primitive.positions {
                let world_position = transform.transform_point(Vec3::from(*position));
                bounds_min = bounds_min.min(world_position);
                bounds_max = bounds_max.max(world_position);
            }
        }
    }

    let root = commands
        .spawn((
            Name::new("Imported Mesh"),
            Transform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ImportedMesh,
            ImportedMeshRoot { id: mesh_id },
        ))
        .id();

    let mut parts = Vec::with_capacity(scene.objects.len());

    for (part_index, object) in scene.objects.iter().enumerate() {
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
                Visibility::Visible,
                InheritedVisibility::default(),
                ImportedMesh,
                ImportedMeshPart {
                    mesh_id,
                    part_index,
                },
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
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ImportedMesh,
                    ImportedMeshPart {
                        mesh_id,
                        part_index,
                    },
                ));
            });
        }

        parts.push(MeshPartEntry {
            index: part_index,
            name: object.name.clone(),
            visible: true,
            entity: Some(object_entity),
        });
    }

    let bounds_center = if bounds_min.x.is_finite() {
        (bounds_min + bounds_max) * 0.5
    } else {
        Vec3::ZERO
    };

    let bounds_radius = if bounds_min.x.is_finite() {
        (bounds_max - bounds_center).length()
    } else {
        1.0
    };

    (root, parts, bounds_center, bounds_radius)
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

pub fn despawn_all_meshes(commands: &mut Commands, scene_state: &mut MeshSceneState) {
    for mesh in scene_state.meshes.drain(..) {
        if let Some(root) = mesh.root {
            commands.entity(root).despawn();
        }
    }

    scene_state.next_id = 0;
}

pub fn update_mesh_light(
    mesh_light: Res<MeshLight>,
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
        Transform::from_translation(mesh_light.position).looking_at(mesh_light.target, Vec3::Y);
}

pub fn sync_mesh_visibility(
    scene_state: Res<MeshSceneState>,
    mut queries: ParamSet<(
        Query<(&ImportedMeshRoot, &mut Visibility)>,
        Query<(&ImportedMeshPart, &mut Visibility)>,
    )>,
) {
    // Whole-mesh visibility.
    {
        let mut root_query = queries.p0();

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
    }

    // Part visibility.
    //
    // This query now matches BOTH the part entity and all of its
    // primitive children, so we do not depend on inherited visibility
    // propagation to hide the actual rendered mesh.
    {
        let mut part_query = queries.p1();

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
    mesh_scene_state: Res<MeshSceneState>,
    time: Res<Time>,
    mut mesh_preview: ResMut<MeshPreview>,
    mut mesh_light: ResMut<MeshLight>,
    cameras: Query<(&Camera, &GlobalTransform), With<MeshPreviewCamera>>,
    mut camera_transforms: Query<&mut Transform, With<MeshPreviewCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let cursor_inside = cursor_position.x >= mesh_viewport.x
        && cursor_position.x <= mesh_viewport.x + mesh_viewport.width
        && cursor_position.y >= mesh_viewport.y
        && cursor_position.y <= mesh_viewport.y + mesh_viewport.height;

    if !cursor_inside {
        return;
    }

    if mesh_input.pointer_over_ui {
        return;
    }

    /*
     * ------------------------------------------------------------
     * Mouse motion
     * ------------------------------------------------------------
     *
     * Left mouse:
     *     Orbit camera
     *
     * Middle mouse:
     *     Pan camera
     *
     * Right mouse:
     *     Also orbit camera
     *
     * Keeping the actual orbit state as yaw/pitch avoids accumulating
     * rotations on the camera Transform itself.
     */

    let mut mouse_delta = Vec2::ZERO;

    for event in mouse_motion.read() {
        mouse_delta += event.delta;
    }

    if mouse_delta != Vec2::ZERO {
        let orbit_button =
            mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.pressed(MouseButton::Right);

        if orbit_button {
            let sensitivity = 0.01;

            mesh_preview.yaw -= mouse_delta.x * sensitivity;
            mesh_preview.pitch -= mouse_delta.y * sensitivity;

            // Prevent the camera from going over the poles.
            mesh_preview.pitch = mesh_preview.pitch.clamp(-1.5, 1.5);
        }

        /*
         * Middle-mouse panning.
         *
         * Pan relative to the camera's current orientation rather
         * than world X/Z, so panning remains intuitive after orbiting.
         */
        if mouse_buttons.pressed(MouseButton::Middle) {
            let Ok((_camera, camera_global)) = cameras.single() else {
                return;
            };

            let right = *camera_global.right();
            let up = *camera_global.up();

            let pan_speed = mesh_preview.distance * 0.0015;

            mesh_preview.target += (-right * mouse_delta.x + up * mouse_delta.y) * pan_speed;
        }
    }

    /*
     * ------------------------------------------------------------
     * Mouse wheel zoom
     * ------------------------------------------------------------
     *
     * Ctrl + wheel:
     *     Zoom
     *
     * Normal wheel:
     *     Pan vertically.
     *
     * This preserves the existing scroll behavior while allowing
     * the speed to come from Editor Settings.
     */

    let mut scroll = 0.0;

    for event in mouse_wheel.read() {
        scroll += event.y;
    }

    if scroll != 0.0 {
        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            let zoom_speed = editor_ui.editor_settings.mouse_scroll_speed;

            // Multiplicative zoom keeps the feel consistent at all distances.
            let zoom_factor = (-scroll * zoom_speed * 0.05).exp();

            mesh_preview.target_distance *= zoom_factor;

            mesh_preview.target_distance = mesh_preview.target_distance.clamp(0.01, 100_000.0);
        } else {
            let Ok((_, camera_global)) = cameras.single() else {
                return;
            };

            let up = *camera_global.up();

            let pan_speed = mesh_preview.distance * 0.05;

            mesh_preview.target += up * scroll * pan_speed;
        }
    }

    /*
     * ------------------------------------------------------------
     * Smooth zoom
     * ------------------------------------------------------------
     */

    let zoom_smoothing = 14.0;

    let t = 1.0 - (-zoom_smoothing * time.delta_secs()).exp();

    mesh_preview.distance = mesh_preview.distance.lerp(mesh_preview.target_distance, t);

    /*
     * ------------------------------------------------------------
     * Light follows mouse position on the actual mesh
     * ------------------------------------------------------------
     */

    if mesh_light.follow_camera {
        if let Ok((_, camera_global)) = cameras.single() {
            let light_distance = mesh_scene_state
                .visible_bounds()
                .map(|(_, radius)| (radius * 2.0).max(1.0))
                .unwrap_or(1.0);

            mesh_light.position = mesh_preview.target - *camera_global.forward() * light_distance;

            mesh_light.target = mesh_preview.target;
        }
    }

    /*
     * ------------------------------------------------------------
     * Apply spherical orbit to the camera
     * ------------------------------------------------------------
     */

    let Ok(mut camera_transform) = camera_transforms.single_mut() else {
        return;
    };

    let rotation = Quat::from_euler(EulerRot::YXZ, mesh_preview.yaw, mesh_preview.pitch, 0.0);

    let offset = rotation * Vec3::new(0.0, 0.0, mesh_preview.distance);

    camera_transform.translation = mesh_preview.target + offset;

    camera_transform.look_at(mesh_preview.target, Vec3::Y);
}
