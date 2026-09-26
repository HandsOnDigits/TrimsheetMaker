pub mod import;

mod regions;

pub mod viewport;

use bevy::{camera::RenderTarget, prelude::*, render::render_resource::TextureFormat};
use bevy_egui::EguiPrimaryContextPass;

#[derive(Component)]
pub struct MeshPreviewCamera;

#[derive(Component)]
pub struct MeshPreviewLight;

#[derive(Resource, Default)]
pub struct MeshViewportInput {
    pub pointer_over_ui: bool,
}

#[derive(Resource, Default)]
pub struct MeshSceneState {
    pub root: Option<Entity>,
    pub bounds_center: Vec3,
    pub bounds_radius: f32,
}

#[derive(Resource)]
pub struct MeshLight {
    pub preset: MeshLightPreset,
    pub position: Vec3,
    pub target: Vec3,
    pub follow_camera: bool,
}

impl Default for MeshLight {
    fn default() -> Self {
        Self {
            preset: MeshLightPreset::Studio,
            position: Vec3::new(2.0, 2.0, 2.0),
            target: Vec3::ZERO,
            follow_camera: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshLightPreset {
    Studio,
    Neutral,
    Warm,
    Cool,
    Flat,
    None,
}

impl MeshLightPreset {
    pub const ALL: [Self; 6] = [
        Self::Studio,
        Self::Neutral,
        Self::Warm,
        Self::Cool,
        Self::Flat,
        Self::None,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Studio => "Studio",
            Self::Neutral => "Neutral",
            Self::Warm => "Warm",
            Self::Cool => "Cool",
            Self::Flat => "Flat",
            Self::None => "None",
        }
    }
}

#[derive(Resource)]
pub struct MeshPreview {
    pub camera: Entity,
    pub target: Vec3,
    pub distance: f32,
    pub target_distance: f32,
    pub rotation: Quat,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Resource, Default)]
pub struct MeshViewportLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct MeshPreviewPlugin;

impl Plugin for MeshPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshSceneState>()
            .init_resource::<MeshViewportLayout>()
            .init_resource::<MeshLight>()
            .init_resource::<MeshViewportInput>()
            .add_systems(
                Startup,
                (setup_mesh_preview, viewport::spawn_mesh_viewport).chain(),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    viewport::update_mesh_viewport,
                    viewport::orbit_camera,
                    viewport::update_mesh_light,
                ),
            );
    }
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
        rotation: Quat::from_rotation_x(-0.25),
        yaw: 0.0,
        pitch: 0.3,
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
