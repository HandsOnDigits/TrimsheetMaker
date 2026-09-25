use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

use super::{MeshLight, MeshPreview, MeshPreviewCamera, MeshPreviewLight, MeshViewportLayout};

use crate::ui::EditorUi;

#[derive(Component)]
pub struct MeshViewportSection;

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

    match *mesh_light {
        MeshLight::Studio => {
            light.illuminance = 10_000.0;
            light.color = Color::srgb(1.0, 1.0, 1.0);

            *transform = Transform::from_xyz(2.0, 4.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y);
        }

        MeshLight::Neutral => {
            light.illuminance = 7_000.0;
            light.color = Color::srgb(1.0, 1.0, 1.0);

            *transform = Transform::from_xyz(-2.0, 3.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y);
        }

        MeshLight::Warm => {
            light.illuminance = 8_000.0;
            light.color = Color::srgb(1.0, 0.78, 0.58);

            *transform = Transform::from_xyz(3.0, 4.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y);
        }

        MeshLight::Cool => {
            light.illuminance = 8_000.0;
            light.color = Color::srgb(0.65, 0.8, 1.0);

            *transform = Transform::from_xyz(-3.0, 4.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y);
        }

        MeshLight::Flat => {
            light.illuminance = 3_000.0;
            light.color = Color::srgb(1.0, 1.0, 1.0);

            *transform = Transform::from_xyz(0.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y);
        }

        MeshLight::None => {
            light.illuminance = 0.0;
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
    editor_ui: Res<EditorUi>,
    time: Res<Time>,
    mut mesh_preview: ResMut<MeshPreview>,
    mut cameras: Query<&mut Transform, With<MeshPreviewCamera>>,
) {
    let Ok(window) = windows.single() else {
        mouse_motion.clear();
        mouse_wheel.clear();
        return;
    };

    let cursor_inside = window
        .cursor_position()
        .is_some_and(|position| mesh_viewport.contains(position));

    // Start orbit only when the mouse is inside the 3D viewport.
    if mouse_buttons.just_pressed(MouseButton::Left) && cursor_inside {
        mesh_preview.orbiting = true;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        mesh_preview.orbiting = false;
    }

    // Orbit.
    if mesh_preview.orbiting {
        let mut delta = Vec2::ZERO;

        for event in mouse_motion.read() {
            delta += event.delta;
        }

        let sensitivity = 0.01;

        mesh_preview.yaw -= delta.x * sensitivity;
        mesh_preview.pitch -= delta.y * sensitivity;

        mesh_preview.pitch = mesh_preview.pitch.clamp(-1.5, 1.5);
    } else {
        mouse_motion.clear();
    }

    // Ctrl + mouse wheel = zoom.
    let ctrl_pressed =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if cursor_inside {
        let mut scroll = 0.0;

        for event in mouse_wheel.read() {
            scroll += event.y;
        }

        if scroll != 0.0 && ctrl_pressed {
            let zoom_speed = editor_ui.editor_settings.mouse_scroll_speed;

            mesh_preview.target_distance *= (-scroll * zoom_speed * 0.01).exp();

            mesh_preview.target_distance = mesh_preview.target_distance.clamp(0.25, 100.0);
        }
    } else {
        mouse_wheel.clear();
    }

    // Smoothly approach the target zoom.
    let smoothing = 12.0;

    let interpolation = 1.0 - (-smoothing * time.delta_secs()).exp();

    mesh_preview.distance = mesh_preview
        .distance
        .lerp(mesh_preview.target_distance, interpolation);

    // Update camera transform.
    let rotation = Quat::from_euler(EulerRot::YXZ, mesh_preview.yaw, mesh_preview.pitch, 0.0);

    let offset = rotation * Vec3::new(0.0, 0.0, mesh_preview.distance);

    for mut transform in &mut cameras {
        transform.translation = mesh_preview.target + offset;

        transform.look_at(mesh_preview.target, Vec3::Y);
    }
}
