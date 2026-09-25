use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    hotkeys::{EditorAction, HotkeyManager},
    mesh::{MeshLight, MeshLightPreset, MeshPreview, MeshViewportInput, MeshViewportLayout},
};

pub fn mesh_panel(
    ui: &mut egui::Ui,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
    mesh_input: &mut MeshViewportInput,
    hotkeys: &HotkeyManager,
    mesh_viewport: &mut MeshViewportLayout,
) {
    ui.horizontal(|ui| {
        ui.heading("Mesh");

        ui.separator();

        if ui.button("−").clicked() {
            mesh_preview.target_distance = (mesh_preview.target_distance * 0.9).clamp(0.25, 100.0);
        }

        ui.label(format!(
            "{:.0}%",
            (4.0 / mesh_preview.target_distance * 100.0).clamp(10.0, 400.0)
        ));

        if ui.button("+").clicked() {
            mesh_preview.target_distance = (mesh_preview.target_distance * 1.1).clamp(0.25, 100.0);
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
                    ui.selectable_value(&mut mesh_light.preset, preset, preset.name());
                }
            });

        ui.separator();

        ui.checkbox(&mut mesh_light.follow_mouse, "Follow Mouse");

        if let Some(hotkey) = hotkeys.display_name(EditorAction::ToggleMeshLightFollowMouse) {
            ui.label(egui::RichText::new(format!("[{hotkey}]")).small().weak());
        }
    });

    // Horizontal split.
    ui.separator();

    // The native Bevy viewport starts below the toolbar.
    let viewport_rect = ui.available_rect_before_wrap();

    mesh_viewport.x = viewport_rect.left();
    mesh_viewport.y = viewport_rect.top();
    mesh_viewport.width = viewport_rect.width();
    mesh_viewport.height = viewport_rect.height();

    let response = ui.interact(
        viewport_rect,
        ui.make_persistent_id("mesh_viewport"),
        egui::Sense::click_and_drag(),
    );

    if response.hovered() {
        mesh_input.pointer_over_ui = false;
    }
}
