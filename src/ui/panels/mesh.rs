use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    hotkeys::{EditorAction, HotkeyManager},
    mesh::{
        MeshLight, MeshLightPreset, MeshPreview, MeshSceneState, MeshViewportInput,
        MeshViewportLayout,
    },
};

pub fn mesh_panel(
    ui: &mut egui::Ui,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
    mesh_input: &mut MeshViewportInput,
    mesh_scene_state: &mut MeshSceneState,
    hotkeys: &HotkeyManager,
    mesh_viewport: &mut MeshViewportLayout,
) {
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

    // Mesh list.
    ui.separator();

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Meshes").strong());

        ui.label(
            egui::RichText::new(format!("{} loaded", mesh_scene_state.meshes.len()))
                .small()
                .weak(),
        );
    });

    if mesh_scene_state.meshes.is_empty() {
        ui.label(egui::RichText::new("No meshes imported.").italics().weak());
    } else {
        egui::ScrollArea::vertical()
            .max_height(140.0)
            .show(ui, |ui| {
                for mesh in &mut mesh_scene_state.meshes {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut mesh.visible, "");

                        let name = if mesh.name.is_empty() {
                            mesh.path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("Mesh")
                        } else {
                            &mesh.name
                        };

                        ui.label(name);
                    });
                }
            });
    }

    // Horizontal split.
    ui.separator();

    // The native Bevy viewport starts below the toolbar and mesh list.
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
