use bevy::prelude::*;
use bevy_egui::egui;

use crate::{mesh::MeshSceneState, ui::EditorUi};

pub fn meshes_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    commands: &mut Commands,
    mesh_scene_state: &mut MeshSceneState,
) {
    if !editor_ui.show_meshes_window {
        return;
    }

    let mut open = editor_ui.show_meshes_window;

    egui::Window::new("Meshes")
        .open(&mut open)
        .resizable(true)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Meshes").strong());

                ui.label(
                    egui::RichText::new(format!("{} loaded", mesh_scene_state.meshes.len()))
                        .small()
                        .weak(),
                );
            });

            ui.separator();

            if mesh_scene_state.meshes.is_empty() {
                ui.label(egui::RichText::new("No meshes imported.").italics().weak());
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for mesh in &mut mesh_scene_state.meshes {
                    let mesh_id = mesh.id;
                    let selected = editor_ui.selected_mesh == Some(mesh_id);
                    let expanded = editor_ui.expanded_meshes.contains(&mesh_id);

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut mesh.visible, "");

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

            ui.separator();

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
                    }

                    editor_ui.expanded_meshes.remove(&selected_id);
                }
            }
        });

    editor_ui.show_meshes_window = open;
}
