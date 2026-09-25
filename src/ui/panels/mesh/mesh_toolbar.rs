use bevy::prelude::*;
use bevy_egui::egui;

use crate::mesh::{MeshLight, MeshLightPreset, MeshPreview, MeshViewportInput};

pub fn mesh_toolbar(
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
                            mesh_preview.yaw = 0.0;
                            mesh_preview.pitch = 0.25;
                            mesh_preview.orbiting = false;

                            mesh_light.yaw = 45.0_f32.to_radians();
                            mesh_light.pitch = 45.0_f32.to_radians();
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
                    });

                    ui.separator();

                    ui.label("Light direction");

                    light_joystick(ui, mesh_light, mesh_input);
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

    let yaw = mesh_light.yaw;
    let pitch = mesh_light.pitch;

    let x = yaw.sin();
    let y = -pitch.sin();

    let mut handle = center + egui::vec2(x, y) * radius;

    if response.dragged() {
        let pointer = response.interact_pointer_pos().unwrap_or(center);

        let mut offset = pointer - center;

        if offset.length() > radius {
            offset = offset.normalized() * radius;
        }

        handle = center + offset;

        let normalized = offset / radius;

        mesh_light.yaw = normalized
            .x
            .asin()
            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

        mesh_light.pitch = (-normalized.y)
            .asin()
            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    }

    if response.hovered() || response.dragged() {
        mesh_input.pointer_over_ui = true;
    }

    painter.circle_filled(handle, 8.0, ui.visuals().widgets.active.bg_fill);

    painter.circle_stroke(
        handle,
        8.0,
        egui::Stroke::new(1.0, ui.visuals().widgets.active.fg_stroke.color),
    );
}
