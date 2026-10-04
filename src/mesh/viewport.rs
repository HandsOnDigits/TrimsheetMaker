use bevy::{
    camera::RenderTarget,
    input::mouse::{MouseMotion, MouseWheel},
    material::OpaqueRendererMethod,
    pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial},
    prelude::*,
    render::render_resource::{AsBindGroup, Extent3d, TextureFormat},
    shader::ShaderRef,
    tasks::{AsyncComputeTaskPool, Task},
};

use futures_lite::future;

use super::{
    ImportedMeshPart, ImportedMeshRoot, MeshLight, MeshLightPreset, MeshPrimitiveSelection,
    MeshSceneState, create_imported_bevy_mesh,
    unwrap::{MeshUvMode, UnwrappedMeshScene, create_unwrapped_bevy_mesh},
};

use bevy_egui::EguiPrimaryContextPass;

use crate::{
    data::{
        Atlas, AtlasBuildResult, AtlasMaterial, TextureNaming, build_atlas,
        create_atlas_build_input,
    },
    rendering::DefaultTextures,
    transform::Rotation,
    ui::EditorUi,
};

#[derive(Resource)]
pub struct AtlasRefresh {
    pub requested: bool,
    pub task: Option<Task<AtlasBuildResult>>,
    pub generation: u64,

    pub building_image_revision: u64,
    pub building_bake_revision: u64,
    pub building_rendering_revision: u64,
}

impl Default for AtlasRefresh {
    fn default() -> Self {
        Self {
            requested: true,
            task: None,
            generation: 0,

            building_image_revision: u64::MAX,
            building_bake_revision: u64::MAX,
            building_rendering_revision: u64::MAX,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct MeshPreviewExtension {
    #[texture(100)]
    #[sampler(101)]
    pub base_color_texture: Option<Handle<Image>>,

    #[texture(102)]
    #[sampler(103)]
    pub normal_texture: Option<Handle<Image>>,

    #[texture(104)]
    #[sampler(105)]
    pub ambient_occlusion_texture: Option<Handle<Image>>,

    #[texture(106)]
    #[sampler(107)]
    pub roughness_texture: Option<Handle<Image>>,

    #[texture(108)]
    #[sampler(109)]
    pub metallic_texture: Option<Handle<Image>>,
}

impl MaterialExtension for MeshPreviewExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/mesh_preview_material.wgsl".into()
    }

    fn enable_shadows() -> bool {
        true
    }
}

pub type MeshPreviewMaterial = ExtendedMaterial<StandardMaterial, MeshPreviewExtension>;

#[derive(Component)]
pub struct MeshPreviewCamera;

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

#[derive(Component)]
pub struct MeshPreviewLightSource;

#[derive(Resource, Default)]
pub struct MeshPreviewMaterials {
    pub by_atlas_material: std::collections::HashMap<usize, Handle<MeshPreviewMaterial>>,
    pub default: Option<Handle<MeshPreviewMaterial>>,
}

#[derive(Resource)]
pub struct MeshPreviewRenderTarget {
    pub image: Handle<Image>,
    pub width: u32,
    pub height: u32,
}

pub struct MeshPreviewPlugin;

impl Plugin for MeshPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MaterialPlugin::<MeshPreviewMaterial>::default(),))
            .init_resource::<MeshViewportLayout>()
            .init_resource::<MeshLight>()
            .init_resource::<MeshViewportInput>()
            .init_resource::<MeshSceneState>()
            .init_resource::<MeshUvSyncState>()
            .init_resource::<AtlasRefresh>()
            .init_resource::<MeshPreviewMaterials>()
            .add_systems(Startup, (setup_mesh_preview, spawn_mesh_viewport).chain())
            .add_systems(
                EguiPrimaryContextPass,
                (
                    update_mesh_viewport,
                    orbit_camera.after(crate::ui::panels::editor_ui),
                ),
            )
            .add_systems(
                Update,
                (
                    resize_mesh_preview_render_target,
                    sync_mesh_visibility,
                    sync_mesh_uv_mode,
                    ensure_atlas_build_requested,
                    sync_atlas_images,
                    finish_atlas_build,
                    sync_atlas_to_mesh_preview_material.after(finish_atlas_build),
                    sync_mesh_preview_material_assignments
                        .after(sync_atlas_to_mesh_preview_material),
                    update_mesh_light,
                ),
            );
    }
}

pub fn sync_atlas_to_mesh_preview_material(
    atlas: Res<Atlas>,
    editor_ui: Res<EditorUi>,
    default_textures: Res<DefaultTextures>,
    mut materials: ResMut<Assets<MeshPreviewMaterial>>,
    mut preview_materials: ResMut<MeshPreviewMaterials>,
) {
    for atlas_material in &atlas.materials {
        let handle = if let Some(handle) = preview_materials
            .by_atlas_material
            .get(&atlas_material.index)
        {
            handle.clone()
        } else {
            let handle = materials.add(MeshPreviewMaterial {
                base: StandardMaterial {
                    opaque_render_method: OpaqueRendererMethod::Forward,
                    ..default()
                },
                extension: MeshPreviewExtension::default(),
            });

            preview_materials
                .by_atlas_material
                .insert(atlas_material.index, handle.clone());

            handle
        };

        let Some(mut material) = materials.get_mut(&handle) else {
            continue;
        };

        sync_preview_material(
            &mut material.extension,
            atlas_material,
            &atlas,
            &editor_ui.project_settings.texture_naming,
            &default_textures,
        );
    }
}

pub fn sync_mesh_preview_material_assignments(
    atlas: Res<Atlas>,
    preview_materials: Res<MeshPreviewMaterials>,
    mut commands: Commands,
    parts: Query<(Entity, &ImportedMeshPart)>,
) {
    for (entity, part) in parts.iter() {
        let Some(primitive_index) = part.primitive_index else {
            if let Some(handle) = &preview_materials.default {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(handle.clone()));
            }

            continue;
        };

        let Some(atlas_material) = atlas.materials.iter().find(|material| {
            material.has_mesh_assignment(part.mesh_id, part.part_index, primitive_index)
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
            continue;
        };

        commands
            .entity(entity)
            .insert(MeshMaterial3d(handle.clone()));
    }
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
    mut lights: Query<(&mut DirectionalLight, &mut Transform), With<MeshPreviewLightSource>>,
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

fn transform_material_uv(uv: [f32; 2], rotation: Rotation) -> [f32; 2] {
    match rotation {
        Rotation::Deg0 => uv,
        Rotation::Deg90 => [uv[1], 1.0 - uv[0]],
        Rotation::Deg180 => [1.0 - uv[0], 1.0 - uv[1]],
        Rotation::Deg270 => [1.0 - uv[1], uv[0]],
    }
}

pub(crate) fn atlas_uv(uv: [f32; 2], material: &AtlasMaterial, atlas: &Atlas) -> [f32; 2] {
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

pub(crate) fn material_uvs(
    uvs: &[[f32; 2]],
    material: Option<&AtlasMaterial>,
    atlas: &Atlas,
) -> Vec<[f32; 2]> {
    let Some(material) = material else {
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

                let uvs = material_uvs(&unwrapped.uvs, material, &atlas);
                mesh3d.0 = create_unwrapped_bevy_mesh(&mut meshes, unwrapped, uvs);
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
    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        mouse_delta += event.delta;
    }

    let mut scroll = 0.0;
    for event in mouse_wheel.read() {
        scroll += event.y;
    }

    let Ok((_camera, camera_global)) = cameras.single() else {
        return;
    };

    let cursor_inside = windows
        .single()
        .ok()
        .and_then(|w| w.cursor_position())
        .map_or(false, |pos| {
            pos.x >= mesh_viewport.x
                && pos.x <= mesh_viewport.x + mesh_viewport.width
                && pos.y >= mesh_viewport.y
                && pos.y <= mesh_viewport.y + mesh_viewport.height
        });

    let can_interact =
        cursor_inside && !mesh_input.pointer_over_ui && !mesh_input.floating_window_dragging;

    if can_interact {
        if mouse_delta != Vec2::ZERO {
            let orbit_button = mouse_buttons.pressed(MouseButton::Left)
                || mouse_buttons.pressed(MouseButton::Right);

            if orbit_button {
                let sensitivity = 0.01;
                mesh_preview.yaw -= mouse_delta.x * sensitivity;
                mesh_preview.pitch =
                    (mesh_preview.pitch - mouse_delta.y * sensitivity).clamp(-1.5, 1.5);
            }

            if mouse_buttons.pressed(MouseButton::Middle) {
                let right = *camera_global.right();
                let up = *camera_global.up();
                let pan_speed = mesh_preview.distance * 0.0015;

                mesh_preview.target += (-right * mouse_delta.x + up * mouse_delta.y) * pan_speed;
            }
        }

        if scroll != 0.0 {
            if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
                let zoom_speed = editor_ui.editor_settings.mouse_scroll_speed;
                let zoom_factor = (-scroll * zoom_speed * 0.05).exp();

                mesh_preview.target_distance =
                    (mesh_preview.target_distance * zoom_factor).clamp(0.01, 100_000.0);
            } else {
                let up = *camera_global.up();
                let pan_speed = mesh_preview.distance * 0.05;

                mesh_preview.target += up * scroll * pan_speed;
            }
        }
    }

    let zoom_smoothing = 14.0;
    let t = 1.0 - (-zoom_smoothing * time.delta_secs()).exp();
    mesh_preview.distance = mesh_preview.distance.lerp(mesh_preview.target_distance, t);

    if mesh_light.follow_camera {
        let light_distance = mesh_scene_state
            .visible_bounds()
            .map(|(_, radius)| (radius * 2.0).max(1.0))
            .unwrap_or(1.0);

        mesh_light.position = mesh_preview.target - *camera_global.forward() * light_distance;
        mesh_light.target = mesh_preview.target;
    }

    if let Ok(mut camera_transform) = camera_transforms.single_mut() {
        let rotation = Quat::from_euler(EulerRot::YXZ, mesh_preview.yaw, mesh_preview.pitch, 0.0);
        let offset = rotation * Vec3::new(0.0, 0.0, mesh_preview.distance);

        camera_transform.translation = mesh_preview.target + offset;
        camera_transform.look_at(mesh_preview.target, Vec3::Y);
    }
}

pub fn sync_atlas_images(
    atlas: Res<Atlas>,
    editor_ui: Res<EditorUi>,
    images: Res<Assets<Image>>,
    mut refresh: ResMut<AtlasRefresh>,
) {
    if atlas.image_revision != refresh.building_image_revision
        || atlas.bake_revision != refresh.building_bake_revision
        || editor_ui.rendering_revision != refresh.building_rendering_revision
    {
        refresh.requested = true;
    }

    if !refresh.requested || refresh.task.is_some() {
        return;
    }

    let image_revision = atlas.image_revision;
    let bake_revision = atlas.bake_revision;
    let rendering_revision = editor_ui.rendering_revision;

    let input =
        create_atlas_build_input(&atlas, &editor_ui.project_settings.texture_naming, &images);

    refresh.generation = refresh.generation.wrapping_add(1);

    refresh.building_image_revision = image_revision;
    refresh.building_bake_revision = bake_revision;
    refresh.building_rendering_revision = rendering_revision;

    let generation = refresh.generation;

    refresh.task =
        Some(AsyncComputeTaskPool::get().spawn(async move { build_atlas(input, generation) }));
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

pub fn finish_atlas_build(
    mut atlas: ResMut<Atlas>,
    mut images: ResMut<Assets<Image>>,
    mut refresh: ResMut<AtlasRefresh>,
    editor_ui: Res<EditorUi>,
) {
    let Some(mut task) = refresh.task.take() else {
        return;
    };

    let Some(result) = future::block_on(future::poll_once(&mut task)) else {
        refresh.task = Some(task);
        return;
    };

    if result.generation != refresh.generation {
        return;
    }

    for (map, image) in result.maps {
        match atlas.images.get(map).cloned() {
            Some(handle) => {
                if let Some(mut existing) = images.get_mut(&handle) {
                    *existing = image;
                } else {
                    let new_handle = images.add(image);
                    *atlas.images.get_mut(map) = Some(new_handle);
                }
            }

            None => {
                let handle = images.add(image);
                *atlas.images.get_mut(map) = Some(handle);
            }
        }
    }

    let still_current = atlas.image_revision == refresh.building_image_revision
        && atlas.bake_revision == refresh.building_bake_revision
        && editor_ui.rendering_revision == refresh.building_rendering_revision;

    refresh.requested = !still_current;
}

pub fn ensure_atlas_build_requested(atlas: Res<Atlas>, mut refresh: ResMut<AtlasRefresh>) {
    if refresh.task.is_some() {
        return;
    }

    if atlas.images.base_color.is_none()
        && atlas.images.normal.is_none()
        && atlas.images.height.is_none()
        && atlas.images.ambient_occlusion.is_none()
        && atlas.images.roughness.is_none()
        && atlas.images.metallic.is_none()
    {
        refresh.requested = true;
    }
}

fn sync_preview_material(
    extension: &mut MeshPreviewExtension,
    atlas_material: &AtlasMaterial,
    atlas: &Atlas,
    texture_naming: &TextureNaming,
    default_textures: &DefaultTextures,
) {
    let mut has_base_color = false;
    let mut has_normal = false;
    let mut has_ao = false;
    let mut has_roughness = false;
    let mut has_metallic = false;

    for texture in &atlas_material.textures {
        let Some(group_id) = texture.group_id.as_deref() else {
            continue;
        };

        let Some(group) = texture_naming
            .groups
            .iter()
            .find(|group| group.id == group_id)
        else {
            continue;
        };

        let rendering = &group.rendering;

        has_base_color |= rendering.base_color;
        has_normal |= rendering.normal;
        has_ao |= rendering.ambient_occlusion.is_some();
        has_roughness |= rendering.roughness.is_some();
        has_metallic |= rendering.metallic.is_some();
    }

    extension.base_color_texture = if has_base_color {
        atlas.images.base_color.clone()
    } else {
        Some(default_textures.checkerboard.clone())
    };

    extension.normal_texture = if has_normal {
        atlas.images.normal.clone()
    } else {
        Some(default_textures.flat_normal.clone())
    };

    extension.ambient_occlusion_texture = if has_ao {
        atlas.images.ambient_occlusion.clone()
    } else {
        Some(default_textures.white.clone())
    };

    extension.roughness_texture = if has_roughness {
        atlas.images.roughness.clone()
    } else {
        Some(default_textures.white.clone())
    };

    extension.metallic_texture = if has_metallic {
        atlas.images.metallic.clone()
    } else {
        Some(default_textures.black.clone())
    };
}
