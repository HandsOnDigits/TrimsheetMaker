use bevy::{input::mouse::MouseMotion, prelude::*};

use super::{MeshPreview, MeshPreviewCamera, MeshViewportLayout};

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

pub fn orbit_camera(
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mesh_viewport: Res<MeshViewportLayout>,
    mut mesh_preview: ResMut<MeshPreview>,
    mut cameras: Query<&mut Transform, With<MeshPreviewCamera>>,
) {
    let Ok(window) = windows.single() else {
        mouse_motion.clear();
        return;
    };

    let cursor_inside = window
        .cursor_position()
        .is_some_and(|position| mesh_viewport.contains(position));

    if mouse_buttons.just_pressed(MouseButton::Left) && cursor_inside {
        mesh_preview.orbiting = true;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        mesh_preview.orbiting = false;
    }

    if !mesh_preview.orbiting {
        mouse_motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;

    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let sensitivity = 0.01;

    mesh_preview.yaw -= delta.x * sensitivity;
    mesh_preview.pitch -= delta.y * sensitivity;

    mesh_preview.pitch = mesh_preview.pitch.clamp(-1.5, 1.5);

    let rotation = Quat::from_euler(EulerRot::YXZ, mesh_preview.yaw, mesh_preview.pitch, 0.0);

    let offset = rotation * Vec3::new(0.0, 0.0, mesh_preview.distance);

    for mut transform in &mut cameras {
        transform.translation = mesh_preview.target + offset;

        transform.look_at(mesh_preview.target, Vec3::Y);
    }
}
