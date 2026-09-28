use bevy::{
    camera::RenderTarget,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::render_resource::TextureFormat,
};

use super::{
    ImportedMeshPart, ImportedMeshRoot, MeshLight, MeshLightPreset, MeshPrimitiveSelection,
    MeshSceneState, create_bevy_mesh,
    unwrap::{MeshUvMode, UnwrappedMeshScene},
};

use bevy_egui::EguiPrimaryContextPass;

use crate::ui::EditorUi;

#[derive(Component)]
pub struct MeshPreviewCamera;

#[derive(Component)]
pub struct MeshPreviewLight;

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
}

#[derive(Resource, Default)]
pub struct MeshViewportLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

fn setup_mesh_preview(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let image = Image::new_target_texture(1, 1, TextureFormat::Rgba8UnormSrgb, None);

    let image_handle = images.add(image);

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
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(2.0, 4.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        MeshPreviewLight,
    ));

    commands.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.7),
            perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

pub struct MeshPreviewPlugin;

impl Plugin for MeshPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshViewportLayout>()
            .init_resource::<MeshLight>()
            .init_resource::<MeshViewportInput>()
            .init_resource::<MeshSceneState>()
            .add_systems(Startup, (setup_mesh_preview, spawn_mesh_viewport).chain())
            .add_systems(
                EguiPrimaryContextPass,
                (update_mesh_viewport, update_mesh_light),
            )
            .add_systems(
                EguiPrimaryContextPass,
                orbit_camera.after(crate::ui::panels::editor_ui),
            )
            .add_systems(PostUpdate, (sync_mesh_visibility, sync_mesh_uv_mode));
    }
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

pub fn sync_mesh_uv_mode(
    mesh_preview: Res<MeshPreview>,
    mesh_scene_state: Res<MeshSceneState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&ImportedMeshPart, &mut Mesh3d)>,
) {
    if !mesh_preview.is_changed() {
        return;
    }

    let Some(mesh_id) = mesh_preview.selected_mesh_id else {
        return;
    };

    let Some(mesh_entry) = mesh_scene_state.get(mesh_id) else {
        return;
    };

    for (part, mut mesh3d) in &mut query {
        if part.mesh_id != mesh_id {
            continue;
        }

        let Some(primitive_index) = part.primitive_index else {
            continue;
        };

        let Some(object) = mesh_entry.scene.objects.get(part.part_index) else {
            continue;
        };

        let Some(primitive) = object.primitives.get(primitive_index) else {
            continue;
        };

        match mesh_preview.uv_mode {
            MeshUvMode::Imported => {
                mesh3d.0 = create_bevy_mesh(
                    &mut meshes,
                    primitive.positions.clone(),
                    primitive.normals.clone(),
                    primitive.tangents.clone(),
                    primitive.uvs.clone(),
                    primitive.indices.clone(),
                );
            }

            MeshUvMode::Unwrapped => {
                if mesh_preview.unwrapped_mesh_id != Some(mesh_id) {
                    continue;
                }

                let Some(unwrapped_scene) = mesh_preview.unwrapped_scene.as_ref() else {
                    continue;
                };

                let Some(unwrapped_object) = unwrapped_scene.objects.get(part.part_index) else {
                    continue;
                };

                let Some(unwrapped) = unwrapped_object.primitives.get(primitive_index) else {
                    continue;
                };

                mesh3d.0 = create_bevy_mesh(
                    &mut meshes,
                    unwrapped.positions.clone(),
                    unwrapped.normals.clone(),
                    unwrapped.tangents.clone(),
                    unwrapped.uvs.clone(),
                    unwrapped.indices.clone(),
                );
            }
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
    /*
     * ------------------------------------------------------------
     * Consume mouse input first.
     *
     * This is important when the pointer is over a floating egui
     * window. We do not want mouse events to remain queued and then
     * move the camera after the window interaction ends.
     * ------------------------------------------------------------
     */

    let mut mouse_delta = Vec2::ZERO;

    for event in mouse_motion.read() {
        mouse_delta += event.delta;
    }

    let mut scroll = 0.0;

    for event in mouse_wheel.read() {
        scroll += event.y;
    }

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

    /*
     * Floating egui windows and other UI elements own the pointer.
     *
     * Do this AFTER consuming the mouse events so they cannot leak
     * into the next frame.
     */
    if mesh_input.pointer_over_ui || mesh_input.floating_window_dragging {
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
     *     Orbit camera
     */

    if mouse_delta != Vec2::ZERO {
        let orbit_button =
            mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.pressed(MouseButton::Right);

        if orbit_button {
            let sensitivity = 0.01;

            mesh_preview.yaw -= mouse_delta.x * sensitivity;
            mesh_preview.pitch -= mouse_delta.y * sensitivity;

            mesh_preview.pitch = mesh_preview.pitch.clamp(-1.5, 1.5);
        }

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
     * Mouse wheel
     * ------------------------------------------------------------
     *
     * Ctrl + wheel:
     *     Zoom
     *
     * Normal wheel:
     *     Vertical pan
     */

    if scroll != 0.0 {
        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            let zoom_speed = editor_ui.editor_settings.mouse_scroll_speed;

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
     * Light follows camera
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
     * Apply spherical orbit
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
