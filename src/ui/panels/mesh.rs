use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    hotkeys::{EditorAction, HotkeyManager},
    mesh::{
        MeshLight, MeshLightPreset, MeshPreview, MeshSceneState, MeshViewportInput,
        MeshViewportLayout,
    },
    ui::EditorUi,
};

pub fn mesh_panel(
    ui: &mut egui::Ui,
    editor_ui: &mut EditorUi,
    commands: &mut Commands,
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
                    let mesh_id = mesh.id;
                    let selected = editor_ui.selected_mesh == Some(mesh_id);
                    let expanded = editor_ui.expanded_meshes.contains(&mesh_id);

                    ui.horizontal(|ui| {
                        // Whole-mesh visibility.
                        ui.checkbox(&mut mesh.visible, "");

                        // Expand/collapse button.
                        let arrow = if expanded { "▼" } else { "▶" };

                        if ui
                            .add(
                                egui::Button::new(arrow)
                                    .frame(false)
                                    .min_size(egui::vec2(18.0, 18.0)),
                            )
                            .clicked()
                        {
                            if expanded {
                                editor_ui.expanded_meshes.remove(&mesh_id);
                            } else {
                                editor_ui.expanded_meshes.insert(mesh_id);
                            }
                        }

                        let name = if mesh.name.is_empty() {
                            mesh.path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("Mesh")
                        } else {
                            &mesh.name
                        };

                        let response = ui.selectable_label(selected, name);

                        if response.clicked() {
                            editor_ui.selected_mesh = Some(mesh_id);
                        }
                    });

                    if expanded {
                        ui.indent(egui::Id::new(("mesh_parts", mesh_id)), |ui| {
                            for part in &mut mesh.parts {
                                ui.horizontal(|ui| {
                                    ui.add_space(4.0);

                                    ui.checkbox(&mut part.visible, "");

                                    ui.label(egui::RichText::new(&part.name).small());
                                });
                            }
                        });
                    }
                }
            });
    }

    ui.horizontal(|ui| {
        let has_selection = editor_ui
            .selected_mesh
            .and_then(|id| mesh_scene_state.get(id))
            .is_some();

        if ui
            .add_enabled(has_selection, egui::Button::new("Delete Selected"))
            .clicked()
        {
            if let Some(selected_id) = editor_ui.selected_mesh.take() {
                if let Some(mesh) = mesh_scene_state.remove(selected_id) {
                    if let Some(root) = mesh.root {
                        commands.entity(root).despawn();
                    }

                    editor_ui.expanded_meshes.remove(&selected_id);
                }

                // Re-frame the viewport around the remaining visible meshes.
                if let Some((center, radius)) = mesh_scene_state.visible_bounds() {
                    mesh_preview.target = center;

                    let fit_distance = (radius * 2.5).max(0.25);

                    mesh_preview.distance = fit_distance;
                    mesh_preview.target_distance = fit_distance;
                } else {
                    mesh_preview.target = Vec3::ZERO;
                    mesh_preview.distance = 4.0;
                    mesh_preview.target_distance = 4.0;
                }
            }
        }
    });

    // Horizontal split.
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
