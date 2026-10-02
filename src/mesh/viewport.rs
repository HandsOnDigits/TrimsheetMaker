use bevy::{
    camera::RenderTarget,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::render_resource::{AsBindGroup, Extent3d, ShaderType, TextureFormat},
    shader::ShaderRef,
};

use crate::pipeline::{RenderingPipelines, TextureChannel};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct MeshPreviewTextureSource {
    pub channel: u32,
    pub _padding: Vec3,
}

impl Default for MeshPreviewTextureSource {
    fn default() -> Self {
        Self {
            channel: 0,
            _padding: Vec3::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct MeshPreviewGraph {
    pub base_color: Vec4,
    pub normal: Vec4,
    pub height: Vec4,
    pub ambient_occlusion: Vec4,
    pub roughness: Vec4,
    pub metallic: Vec4,
}

impl Default for MeshPreviewGraph {
    fn default() -> Self {
        Self {
            base_color: Vec4::ZERO,
            normal: Vec4::ZERO,
            height: Vec4::ZERO,
            ambient_occlusion: Vec4::ZERO,
            roughness: Vec4::ZERO,
            metallic: Vec4::ZERO,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MeshPreviewMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub base_color: Handle<Image>,

    #[texture(2, filterable = false)]
    pub normal: Handle<Image>,

    #[texture(3, filterable = false)]
    pub height: Handle<Image>,

    #[texture(4, filterable = false)]
    pub ambient_occlusion: Handle<Image>,

    #[texture(5, filterable = false)]
    pub roughness: Handle<Image>,

    #[texture(6, filterable = false)]
    pub metallic: Handle<Image>,

    #[uniform(7)]
    pub graph: MeshPreviewGraph,
}

impl Material for MeshPreviewMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mesh_preview_material.wgsl".into()
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

use super::{
    ImportedMeshPart, ImportedMeshRoot, MeshLight, MeshLightPreset, MeshPrimitiveSelection,
    MeshSceneState, create_imported_bevy_mesh,
    unwrap::{MeshUvMode, UnwrappedMeshScene, create_unwrapped_bevy_mesh},
};

use bevy_egui::EguiPrimaryContextPass;

use crate::{
    data::{Atlas, AtlasMaterial, rebuild_atlas_maps},
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

    pub material: Handle<MeshPreviewMaterial>,
    pub uv_revision: u64,

    // Fallback textures
    pub fallback_base_color: Handle<Image>,
    pub fallback_normal: Handle<Image>,
    pub fallback_height: Handle<Image>,
    pub fallback_ambient_occlusion: Handle<Image>,
    pub fallback_roughness: Handle<Image>,
    pub fallback_metallic: Handle<Image>,
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
pub struct AtlasMaterialSyncState {
    pub atlas_revision: u64,
    pub pipeline_revision: u64,
}

impl Default for AtlasMaterialSyncState {
    fn default() -> Self {
        Self {
            atlas_revision: u64::MAX,
            pipeline_revision: u64::MAX,
        }
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

    let white = create_solid_image(&mut images, [255, 255, 255, 255]);
    let black = create_solid_image(&mut images, [0, 0, 0, 255]);
    let flat_normal = create_solid_image(&mut images, [128, 128, 255, 255]);

    let material = materials.add(MeshPreviewMaterial {
        base_color: white.clone(),
        normal: flat_normal.clone(),
        height: black.clone(),
        ambient_occlusion: white.clone(),
        roughness: white.clone(),
        metallic: black.clone(),
        graph: MeshPreviewGraph::default(),
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

        fallback_base_color: white.clone(),
        fallback_normal: flat_normal,
        fallback_height: black.clone(),
        fallback_ambient_occlusion: white.clone(),
        fallback_roughness: white,
        fallback_metallic: black,

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

fn create_solid_image(images: &mut Assets<Image>, color: [u8; 4]) -> Handle<Image> {
    let image = Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        color.to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );

    images.add(image)
}

fn graph_channel(channel: TextureChannel) -> Vec4 {
    let channel_index = match channel {
        TextureChannel::Red => 0,
        TextureChannel::Green => 1,
        TextureChannel::Blue => 2,
        TextureChannel::Alpha => 3,
    };

    Vec4::new(channel_index as f32, 0.0, 0.0, 0.0)
}

fn texture_for_group(
    group: &str,
    atlas: &Atlas,
    mesh_preview: &MeshPreview,
) -> Option<Handle<Image>> {
    let group = group.trim();

    match group {
        "base_color" => Some(
            atlas
                .images
                .base_color
                .clone()
                .unwrap_or_else(|| mesh_preview.fallback_base_color.clone()),
        ),

        "normal" => Some(
            atlas
                .images
                .normal
                .clone()
                .unwrap_or_else(|| mesh_preview.fallback_normal.clone()),
        ),

        "height" => Some(
            atlas
                .images
                .height
                .clone()
                .unwrap_or_else(|| mesh_preview.fallback_height.clone()),
        ),

        "ambient_occlusion" => Some(
            atlas
                .images
                .ambient_occlusion
                .clone()
                .unwrap_or_else(|| mesh_preview.fallback_ambient_occlusion.clone()),
        ),

        "roughness" => Some(
            atlas
                .images
                .roughness
                .clone()
                .unwrap_or_else(|| mesh_preview.fallback_roughness.clone()),
        ),

        "metallic" => Some(
            atlas
                .images
                .metallic
                .clone()
                .unwrap_or_else(|| mesh_preview.fallback_metallic.clone()),
        ),

        _ => None,
    }
}

pub struct MeshPreviewPlugin;

impl Plugin for MeshPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<MeshPreviewMaterial>::default())
            .init_resource::<MeshViewportLayout>()
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
                    sync_atlas_images,
                    sync_mesh_material.after(sync_atlas_images),
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
        (y + (1.0 - uv[1]) * height) / atlas_height,
    ]
}

fn material_uvs(
    uvs: &[[f32; 2]],
    material: Option<&AtlasMaterial>,
    atlas: &Atlas,
) -> Vec<[f32; 2]> {
    let Some(material) = material else {
        // Map unassigned primitives to a single pixel to avoid rendering the full atlas
        return vec![[0.0, 0.0]; uvs.len()];
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
                let uvs = material_uvs(&primitive.uvs, material, &atlas);

                let handle = create_imported_bevy_mesh(&mut meshes, primitive, uvs);

                mesh3d.0 = handle;
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
    rendering_pipelines: Res<RenderingPipelines>,
    mesh_scene_state: Res<MeshSceneState>,
    mut materials: ResMut<Assets<MeshPreviewMaterial>>,
    mesh_preview: Res<MeshPreview>,
    mut sync_state: ResMut<AtlasMaterialSyncState>,
) {
    let atlas_revision = atlas.image_revision;
    let pipeline_revision = rendering_pipelines.revision;

    if sync_state.atlas_revision == atlas_revision
        && sync_state.pipeline_revision == pipeline_revision
    {
        return;
    }

    let Some(mut material) = materials.get_mut(&mesh_preview.material) else {
        return;
    };

    /*
     * Start with safe fallback/semantic bindings.
     */
    material.base_color = atlas
        .images
        .base_color
        .clone()
        .unwrap_or_else(|| mesh_preview.fallback_base_color.clone());

    material.normal = atlas
        .images
        .normal
        .clone()
        .unwrap_or_else(|| mesh_preview.fallback_normal.clone());

    material.height = atlas
        .images
        .height
        .clone()
        .unwrap_or_else(|| mesh_preview.fallback_height.clone());

    material.ambient_occlusion = atlas
        .images
        .ambient_occlusion
        .clone()
        .unwrap_or_else(|| mesh_preview.fallback_ambient_occlusion.clone());

    material.roughness = atlas
        .images
        .roughness
        .clone()
        .unwrap_or_else(|| mesh_preview.fallback_roughness.clone());

    material.metallic = atlas
        .images
        .metallic
        .clone()
        .unwrap_or_else(|| mesh_preview.fallback_metallic.clone());

    material.graph = MeshPreviewGraph::default();

    /*
     * Find the material belonging to the currently selected primitive.
     */
    let material_ref = mesh_preview.selected_primitive.and_then(|selection| {
        let mesh = mesh_scene_state.get(selection.mesh_id)?;

        let object = mesh.scene.objects.get(selection.object_index)?;

        let primitive = object.primitives.get(selection.primitive_index)?;

        primitive.material.as_ref()
    });

    /*
     * Compile the rendering graph into semantic material bindings.
     */
    if let Some(material_ref) = material_ref {
        if let Some(pipeline) = rendering_pipelines.pipeline_for(material_ref) {
            if let Ok(bindings) = pipeline.compile() {
                apply_rendering_bindings(&mut material, &bindings, &atlas, &mesh_preview);
            }
        }
    }

    sync_state.atlas_revision = atlas_revision;
    sync_state.pipeline_revision = pipeline_revision;
}

fn apply_rendering_bindings(
    material: &mut MeshPreviewMaterial,
    bindings: &crate::pipeline::RenderingMaterialBindings,
    atlas: &Atlas,
    mesh_preview: &MeshPreview,
) {
    /*
     * Base Color
     */
    if let Some(binding) = &bindings.base_color {
        if let Some(texture) = texture_for_group(&binding.texture_group, atlas, mesh_preview) {
            material.base_color = texture;
            material.graph.base_color = Vec4::ZERO;
        }
    }

    /*
     * Normal
     */
    if let Some(binding) = &bindings.normal {
        if let Some(texture) = texture_for_group(&binding.texture_group, atlas, mesh_preview) {
            material.normal = texture;
            material.graph.normal = Vec4::ZERO;
        }
    }

    /*
     * Height
     */
    if let Some(binding) = &bindings.height {
        if let Some(texture) = texture_for_group(&binding.texture_group, atlas, mesh_preview) {
            material.height = texture;
            material.graph.height = graph_channel(binding.channel);
        }
    }

    /*
     * Ambient Occlusion
     */
    if let Some(binding) = &bindings.ambient_occlusion {
        if let Some(texture) = texture_for_group(&binding.texture_group, atlas, mesh_preview) {
            material.ambient_occlusion = texture;
            material.graph.ambient_occlusion = graph_channel(binding.channel);
        }
    }

    /*
     * Roughness
     */
    if let Some(binding) = &bindings.roughness {
        if let Some(texture) = texture_for_group(&binding.texture_group, atlas, mesh_preview) {
            material.roughness = texture;
            material.graph.roughness = graph_channel(binding.channel);
        }
    }

    /*
     * Metallic
     */
    if let Some(binding) = &bindings.metallic {
        if let Some(texture) = texture_for_group(&binding.texture_group, atlas, mesh_preview) {
            material.metallic = texture;
            material.graph.metallic = graph_channel(binding.channel);
        }
    }
}

pub fn sync_atlas_images(
    mut atlas: ResMut<Atlas>,
    mut images: ResMut<Assets<Image>>,
    mut rendering_pipelines: ResMut<RenderingPipelines>,
    mut sync_state: ResMut<AtlasImageSyncState>,
) {
    if sync_state.revision == atlas.image_revision {
        return;
    }

    let revision_before = atlas.image_revision;

    println!(
        "SYNC ATLAS IMAGES: {} -> {}",
        sync_state.revision, revision_before
    );

    rebuild_atlas_maps(&mut atlas, &mut images, &mut rendering_pipelines);

    println!(
        "SYNC ATLAS IMAGES AFTER REBUILD: {} -> {}",
        revision_before, atlas.image_revision
    );

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
