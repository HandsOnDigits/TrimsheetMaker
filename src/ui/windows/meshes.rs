use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    data::Atlas,
    mesh::{MeshPrimitiveSelection, MeshSceneState, viewport::MeshPreview},
    ui::EditorUi,
};

use super::FloatingWindowInteraction;

pub fn meshes_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    commands: &mut Commands,
    mesh_scene_state: &mut MeshSceneState,
    mesh_preview: &mut MeshPreview,
) -> bool {
    let mut open = editor_ui.show_meshes_window;

    let window_response = egui::Window::new("Meshes")
        .open(&mut open)
        .resizable(true)
        .default_width(300.0)
        .default_pos(egui::pos2(
            ctx.viewport_rect().center().x - 150.0,
            ctx.viewport_rect().center().y - 200.0,
        ))
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(16.0));

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

                        let arrow = if expanded { "v" } else { ">" };

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
                            mesh_preview.selected_mesh_id = Some(mesh_id);
                            mesh_preview.selected_primitive = None;
                        }
                    });

                    if expanded {
                        ui.indent(egui::Id::new(("mesh_parts", mesh_id)), |ui| {
                            for part in &mut mesh.parts {
                                let object_index = part.object_index;

                                ui.horizontal(|ui| {
                                    ui.add_space(4.0);

                                    ui.checkbox(&mut part.visible, "");

                                    ui.label(egui::RichText::new(&part.name).small());
                                });

                                let Some(object) = mesh.scene.objects.get(object_index) else {
                                    continue;
                                };

                                ui.indent(
                                    egui::Id::new(("mesh_primitives", mesh_id, object_index)),
                                    |ui| {
                                        for primitive_index in 0..object.primitives.len() {
                                            let selected = mesh_preview.selected_primitive
                                                == Some(MeshPrimitiveSelection {
                                                    mesh_id,
                                                    object_index,
                                                    primitive_index,
                                                });

                                            let label = format!("Primitive {primitive_index}");

                                            if ui.selectable_label(selected, label).clicked() {
                                                mesh_preview.selected_mesh_id = Some(mesh_id);

                                                mesh_preview.selected_primitive =
                                                    Some(MeshPrimitiveSelection {
                                                        mesh_id,
                                                        object_index,
                                                        primitive_index,
                                                    });
                                            }
                                        }
                                    },
                                );
                            }
                        });
                    }
                }
            });

            ui.separator();

            let can_assign =
                editor_ui.selected_material.is_some() && mesh_preview.selected_primitive.is_some();

            if let Some(material_index) = editor_ui.selected_material {
                if let Some(material) = atlas
                    .materials
                    .iter()
                    .find(|material| material.index == material_index)
                {
                    ui.label(format!("Material: {}", material.name));
                }
            } else {
                ui.label("Select a material first");
            }

            if mesh_preview.selected_primitive.is_none() {
                ui.label("Select a mesh first");
            }

            let is_assigned = if let (Some(material_index), Some(selection)) =
                (editor_ui.selected_material, mesh_preview.selected_primitive)
            {
                atlas
                    .materials
                    .iter()
                    .find(|material| material.index == material_index)
                    .is_some_and(|material| {
                        material.has_mesh_assignment(
                            selection.mesh_id,
                            selection.object_index,
                            selection.primitive_index,
                        )
                    })
            } else {
                false
            };

            ui.horizontal(|ui| {
                if is_assigned {
                    ui.label(
                        egui::RichText::new("Assigned")
                            .strong()
                            .color(egui::Color32::LIGHT_GREEN),
                    );

                    if ui.button("Unassign").clicked() {
                        if let (Some(material_index), Some(selection)) =
                            (editor_ui.selected_material, mesh_preview.selected_primitive)
                        {
                            if let Some(material) = atlas
                                .materials
                                .iter_mut()
                                .find(|material| material.index == material_index)
                            {
                                let material_ref = mesh_scene_state
                                    .get(selection.mesh_id)
                                    .and_then(|mesh| mesh.scene.objects.get(selection.object_index))
                                    .and_then(|object| {
                                        object.primitives.get(selection.primitive_index)
                                    })
                                    .and_then(|primitive| primitive.material.clone());

                                material.assign_mesh(
                                    selection.mesh_id,
                                    selection.object_index,
                                    selection.primitive_index,
                                );

                                if material.material_ref.is_none() {
                                    material.material_ref = material_ref;
                                }
                            }
                        }
                    }
                } else {
                    let response =
                        ui.add_enabled(can_assign, egui::Button::new("Assign to Material"));

                    if response.clicked() {
                        if let (Some(material_index), Some(selection)) =
                            (editor_ui.selected_material, mesh_preview.selected_primitive)
                        {
                            // Remove this primitive from any previous material.
                            for material in &mut atlas.materials {
                                material.unassign_mesh(
                                    selection.mesh_id,
                                    selection.object_index,
                                    selection.primitive_index,
                                );
                            }

                            atlas.uv_revision = atlas.uv_revision.wrapping_add(1);

                            // Assign it to the selected material.
                            if let Some(material) = atlas
                                .materials
                                .iter_mut()
                                .find(|material| material.index == material_index)
                            {
                                material.assign_mesh(
                                    selection.mesh_id,
                                    selection.object_index,
                                    selection.primitive_index,
                                );
                            }

                            atlas.uv_revision = atlas.uv_revision.wrapping_add(1);
                        }
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

                    for material in &mut atlas.materials {
                        material
                            .mesh_assignments
                            .retain(|assignment| assignment.mesh_id != selected_id);
                    }

                    atlas.uv_revision = atlas.uv_revision.wrapping_add(1);

                    if mesh_preview.selected_mesh_id == Some(selected_id) {
                        mesh_preview.selected_mesh_id = None;
                        mesh_preview.selected_primitive = None;
                    }

                    editor_ui.expanded_meshes.remove(&selected_id);
                }
            }
        });

    let owns_pointer = window_response
        .as_ref()
        .map(|response| response.owns_pointer(ctx))
        .unwrap_or(false);

    editor_ui.show_meshes_window = open;

    owns_pointer
}
