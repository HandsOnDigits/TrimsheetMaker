use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    hotkeys::{EditorAction, HotkeyManager},
    mesh::{MeshLight, MeshLightPreset, MeshPreview, MeshViewportInput},
};

pub fn mesh_panel(
    ctx: &egui::Context,
    rect: egui::Rect,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
    mesh_input: &mut MeshViewportInput,
    hotkeys: &HotkeyManager,
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

                            mesh_light.position = Vec3::new(3.0, 3.0, 4.0);
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

                        ui.checkbox(&mut mesh_light.follow_mouse, "Follow Mouse");

                        if let Some(hotkey) =
                            hotkeys.display_name(EditorAction::ToggleMeshLightFollowMouse)
                        {
                            ui.label(egui::RichText::new(format!("[{hotkey}]")).small().weak());
                        }
                    });
                });

            if frame_response.response.hovered() || frame_response.response.contains_pointer() {
                mesh_input.pointer_over_ui = true;
            }
        });
}
