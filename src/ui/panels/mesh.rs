use bevy::prelude::*;
use bevy_egui::egui;

use crate::mesh::{MeshLight, MeshLightPreset, MeshPreview, MeshViewportInput};

pub fn mesh_panel(
    ctx: &egui::Context,
    rect: egui::Rect,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
    mesh_input: &mut MeshViewportInput,
) {
    egui::Area::new("mesh_viewport_toolbar".into())
        .order(egui::Order::Foreground)
        .fixed_pos(rect.left_top() + egui::vec2(8.0, 8.0))
        .show(ctx, |ui| {
            let frame_response = egui::Frame::menu(ui.style())
                .inner_margin(egui::Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("−").clicked() {
                            mesh_preview.target_distance =
                                (mesh_preview.target_distance * 0.9).clamp(0.25, 100.0);
                        }

                        ui.label(format!(
                            "{:.0}%",
                            (4.0 / mesh_preview.target_distance * 100.0).clamp(10.0, 400.0)
                        ));

                        if ui.button("+").clicked() {
                            mesh_preview.target_distance =
                                (mesh_preview.target_distance * 1.1).clamp(0.25, 100.0);
                        }

                        ui.separator();

                        if ui.button("Reset").clicked() {
                            mesh_preview.target = Vec3::ZERO;
                            mesh_preview.distance = 4.0;
                            mesh_preview.target_distance = 4.0;
                            mesh_preview.rotation = Quat::from_rotation_x(-0.25);
                            mesh_preview.orbiting = false;

                            mesh_light.rotation = Quat::from_euler(
                                EulerRot::YXZ,
                                45.0_f32.to_radians(),
                                45.0_f32.to_radians(),
                                0.0,
                            );
                        }

                        ui.separator();

                        ui.label("Light");

                        egui::ComboBox::from_id_salt("mesh_light_selector")
                            .selected_text(mesh_light.preset.name())
                            .show_ui(ui, |ui| {
                                for preset in MeshLightPreset::ALL {
                                    ui.selectable_value(
                                        &mut mesh_light.preset,
                                        preset,
                                        preset.name(),
                                    );
                                }
                            });

                        ui.separator();

                        egui::ComboBox::from_id_salt("light_direction_dropdown")
                            .selected_text("Light Direction")
                            .show_ui(ui, |ui| {
                                light_joystick(ui, mesh_light, mesh_input);
                            });
                    });
                });

            if frame_response.response.hovered() || frame_response.response.contains_pointer() {
                mesh_input.pointer_over_ui = true;
            }
        });
}

fn light_joystick(
    ui: &mut egui::Ui,
    mesh_light: &mut MeshLight,
    mesh_input: &mut MeshViewportInput,
) {
    const SIZE: f32 = 100.0;

    let (rect, response) = ui.allocate_exact_size(egui::vec2(SIZE, SIZE), egui::Sense::drag());

    let painter = ui.painter_at(rect);

    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.5 - 6.0;

    painter.circle(
        center,
        radius,
        ui.visuals().extreme_bg_color,
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
    );

    painter.line_segment(
        [
            egui::pos2(center.x - radius, center.y),
            egui::pos2(center.x + radius, center.y),
        ],
        egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
    );

    painter.line_segment(
        [
            egui::pos2(center.x, center.y - radius),
            egui::pos2(center.x, center.y + radius),
        ],
        egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
    );

    let mut handle_offset = egui::Vec2::ZERO;

    // Read the actual current mouse position.
    if response.dragged() {
        if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
            let offset = pointer_pos - center;

            handle_offset = if offset.length_sq() > radius * radius {
                offset.normalized() * radius
            } else {
                offset
            };

            let horizontal = handle_offset.x / radius;
            let vertical = -handle_offset.y / radius;

            let yaw = horizontal * std::f32::consts::PI;
            let pitch = vertical * (std::f32::consts::FRAC_PI_2);

            let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);

            mesh_light.rotation = rotation;

            mesh_input.pointer_over_ui = true;
        }
    }

    if response.hovered() || response.dragged() {
        mesh_input.pointer_over_ui = true;
    }

    // Calculate the handle position from the actual light direction
    // when we aren't actively dragging.
    if !response.dragged() {
        let direction = mesh_light.rotation * Vec3::NEG_Z;

        handle_offset = egui::vec2(direction.x, -direction.y);

        if handle_offset.length_sq() > 1.0 {
            handle_offset = handle_offset.normalized();
        }

        handle_offset *= radius;
    }

    let handle = center + handle_offset;

    painter.circle_filled(handle, 8.0, ui.visuals().widgets.active.bg_fill);

    painter.circle_stroke(
        handle,
        8.0,
        egui::Stroke::new(1.0, ui.visuals().widgets.active.fg_stroke.color),
    );
}
