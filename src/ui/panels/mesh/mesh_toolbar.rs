use bevy::prelude::*;
use bevy_egui::egui;

use crate::mesh::{MeshLight, MeshPreview};

pub fn mesh_toolbar(
    ctx: &egui::Context,
    rect: egui::Rect,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
) {
    egui::Area::new("mesh_viewport_toolbar".into())
        .order(egui::Order::Foreground)
        .fixed_pos(rect.left_top() + egui::vec2(8.0, 8.0))
        .show(ctx, |ui| {
            egui::Frame::menu(ui.style())
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
                        }

                        ui.separator();

                        ui.label("Light");

                        egui::ComboBox::from_id_salt("mesh_light_selector")
                            .selected_text(mesh_light.name())
                            .show_ui(ui, |ui| {
                                for light in MeshLight::ALL {
                                    ui.selectable_value(mesh_light, light, light.name());
                                }
                            });
                    });
                });
        });
}
