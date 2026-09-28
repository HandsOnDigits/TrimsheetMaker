use bevy::{
    camera::RenderTarget,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
};

use super::{
    ImportedMeshPart, ImportedMeshRoot, MeshLight, MeshLightPreset, MeshPrimitiveSelection,
    MeshSceneState,
    unwrap::{MeshUvMode, UnwrappedMeshScene, create_unwrapped_bevy_mesh},
};

use bevy_egui::EguiPrimaryContextPass;

use crate::{
    data::{Atlas, AtlasMaterial, rebuild_atlas_image},
    transform::Rotation,
    ui::EditorUi,
};

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

    pub material: Handle<StandardMaterial>,
    pub uv_revision: u64,
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

#[derive(Resource)]
pub struct AtlasImageSyncState {
    pub revision: u64,
}

impl Default for AtlasImageSyncState {
    fn default() -> Self {
        Self { revision: u64::MAX }
    }
}

#[derive(Resource)]
pub struct AtlasMaterialSyncState {
    pub revision: u64,
}

impl Default for AtlasMaterialSyncState {
    fn default() -> Self {
        Self { revision: u64::MAX }
    }
}

#[derive(Resource)]
pub struct MeshPreviewRenderTarget {
    pub image: Handle<Image>,
    pub width: u32,
    pub height: u32,
}

fn setup_mesh_preview(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.7,
        ..default()
    });

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
        MeshPreviewLight,
    ));
}

pub struct MeshPreviewPlugin;

impl Plugin for MeshPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshViewportLayout>()
            .init_resource::<MeshLight>()
            .init_resource::<MeshViewportInput>()
            .init_resource::<MeshSceneState>()
            .init_resource::<MeshUvSyncState>()
            .init_resource::<AtlasImageSyncState>()
            .init_resource::<AtlasMaterialSyncState>()
            .add_systems(Startup, (setup_mesh_preview, spawn_mesh_viewport).chain())
            .add_systems(
                EguiPrimaryContextPass,
                (
                    update_mesh_viewport,
                    update_mesh_light,
                    orbit_camera.after(crate::ui::panels::editor_ui),
                ),
            )
            .add_systems(
                Update,
                (
                    resize_mesh_preview_render_target,
                    sync_mesh_visibility,
                    sync_mesh_uv_mode,
                    sync_atlas_image,
                    sync_mesh_material.after(sync_atlas_image),
                ),
            );
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

fn transform_material_uv(uv: [f32; 2], rotation: Rotation) -> [f32; 2] {
    match rotation {
        Rotation::Deg0 => uv,
        Rotation::Deg90 => [uv[1], 1.0 - uv[0]],
        Rotation::Deg180 => [1.0 - uv[0], 1.0 - uv[1]],
        Rotation::Deg270 => [1.0 - uv[1], uv[0]],
    }
}

fn atlas_uv(uv: [f32; 2], material: &AtlasMaterial, atlas: &Atlas) -> [f32; 2] {
    let uv = transform_material_uv(uv, material.transform.rotation);

    let atlas_width = atlas.width as f32;
    let atlas_height = atlas.height as f32;

    if atlas_width <= 0.0 || atlas_height <= 0.0 {
        return [0.0, 0.0];
    }

    let x = material.transform.x as f32;
    let y = material.transform.y as f32;
    let width = material.transform.width as f32;
    let height = material.transform.height as f32;

    [
        (x + uv[0] * width) / atlas_width,
        1.0 - ((y + height - uv[1] * height) / atlas_height),
    ]
}

fn material_uvs(
    uvs: &[[f32; 2]],
    material: Option<&AtlasMaterial>,
    atlas: &Atlas,
) -> Vec<[f32; 2]> {
    let Some(material) = material else {
        return uvs.to_vec();
    };

    uvs.iter()
        .copied()
        .map(|uv| atlas_uv(uv, material, atlas))
        .collect()
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
                let Some(source_mesh) = part.source_mesh.clone() else {
                    continue;
                };

                if mesh3d.0 != source_mesh {
                    mesh3d.0 = source_mesh.clone();
                }

                let uvs = material_uvs(&primitive.uvs, material, &atlas);

                if let Some(mut mesh) = meshes.get_mut(&source_mesh) {
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                }
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

                let uvs = material_uvs(&unwrapped.uvs, material, &atlas);

                let handle = create_unwrapped_bevy_mesh(&mut meshes, unwrapped, uvs);

                mesh3d.0 = handle;
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

pub fn sync_mesh_material(
    atlas: Res<Atlas>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_preview: Res<MeshPreview>,
    mut sync_state: ResMut<AtlasMaterialSyncState>,
) {
    let Some(image) = atlas.image.clone() else {
        return;
    };

    let Some(mut material) = materials.get_mut(&mesh_preview.material) else {
        return;
    };

    if sync_state.revision == atlas.image_revision
        && material.base_color_texture.as_ref() == Some(&image)
    {
        return;
    }

    material.base_color = Color::WHITE;
    material.base_color_texture = Some(image);

    sync_state.revision = atlas.image_revision;
}

pub fn sync_atlas_image(
    mut atlas: ResMut<Atlas>,
    mut images: ResMut<Assets<Image>>,
    mut sync_state: ResMut<AtlasImageSyncState>,
) {
    if sync_state.revision == atlas.image_revision {
        return;
    }

    rebuild_atlas_image(&mut atlas, &mut images, Some("base_color"));

    sync_state.revision = atlas.image_revision;
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
