use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    hotkeys::{EditorAction, HotkeyManager},
    mesh::{
        MeshLight, MeshLightPreset,
        viewport::{MeshPreview, MeshViewportInput, MeshViewportLayout},
    },
};

pub fn mesh_panel(
    ui: &mut egui::Ui,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
    mesh_input: &mut MeshViewportInput,
    hotkeys: &HotkeyManager,
    mesh_viewport: &mut MeshViewportLayout,
) {
    // ------------------------------------------------------------
    // Mesh toolbar
    // ------------------------------------------------------------

    ui.horizontal(|ui| {
        ui.heading("Mesh");

        ui.separator();

        ui.label("Light");

        egui::ComboBox::from_id_salt("mesh_light_selector")
            .selected_text(mesh_light.preset.name())
            .show_ui(ui, |ui| {
                for preset in MeshLightPreset::ALL {
                    ui.selectable_value(&mut mesh_light.preset, preset, preset.name());
                }
            });

        ui.checkbox(&mut mesh_light.follow_camera, "Follow Camera");

        if let Some(hotkey) = hotkeys.display_name(EditorAction::ToggleMeshLightFollowCamera) {
            ui.label(egui::RichText::new(format!("[{hotkey}]")).small().weak());
        }

        ui.separator();

        if ui.button("Reset").clicked() {
            mesh_preview.target = Vec3::ZERO;
            mesh_preview.distance = 4.0;
            mesh_preview.target_distance = 4.0;
            mesh_preview.yaw = 0.0;
            mesh_preview.pitch = 0.3;

            mesh_light.position = Vec3::new(3.0, 3.0, 4.0);
            mesh_light.target = Vec3::ZERO;
        }
    });

    // ------------------------------------------------------------
    // Viewport
    // ------------------------------------------------------------

    ui.separator();

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
