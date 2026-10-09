use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

use super::{MeshPreview, MeshPreviewCamera, MeshViewportInput, MeshViewportLayout};

use crate::{
    mesh::{MeshLight, MeshSceneState},
    ui::EditorUi,
};

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
        .is_some_and(|pos| {
            pos.x >= mesh_viewport.x
                && pos.x <= mesh_viewport.x + mesh_viewport.width
                && pos.y >= mesh_viewport.y
                && pos.y <= mesh_viewport.y + mesh_viewport.height
        });

    let can_interact =
        cursor_inside && !mesh_input.pointer_over_ui && !mesh_input.floating_window_dragging;

    if can_interact {
        // Camera orbit.
        if mouse_delta != Vec2::ZERO {
            let orbit_button = mouse_buttons.pressed(MouseButton::Left)
                || mouse_buttons.pressed(MouseButton::Right);

            if orbit_button {
                let sensitivity = 0.01;

                mesh_preview.yaw -= mouse_delta.x * sensitivity;

                mesh_preview.pitch =
                    (mesh_preview.pitch - mouse_delta.y * sensitivity).clamp(-1.5, 1.5);
            }

            // Camera pan.
            if mouse_buttons.pressed(MouseButton::Middle) {
                let right = *camera_global.right();
                let up = *camera_global.up();
                let pan_speed = mesh_preview.distance * 0.0015;

                mesh_preview.target += (-right * mouse_delta.x + up * mouse_delta.y) * pan_speed;
            }
        }

        // Camera zoom / vertical pan.
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

        let view_direction = (mesh_preview.target - camera_transform.translation).normalize();

        let up = if view_direction.dot(Vec3::Y).abs() > 0.98 {
            Vec3::Z
        } else {
            Vec3::Y
        };

        camera_transform.look_at(mesh_preview.target, up);
    }
}
