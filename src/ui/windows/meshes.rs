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

            /*
             * Atlas assignment
             *
             * The selected mesh primitive is independent from the atlas
             * region being inspected. The inspected atlas region is the
             * target of an assignment; it is not a material selection.
             */
            let selected_primitive = mesh_preview.selected_primitive;

            let inspected_region = editor_ui.inspected_atlas_region.and_then(|index| {
                atlas
                    .materials
                    .iter()
                    .find(|material| material.index == index)
            });

            if let Some(region) = inspected_region {
                ui.label(format!("Atlas Region: {}", region.name));
            } else {
                ui.label("Inspect an atlas region first");
            }

            if selected_primitive.is_none() {
                ui.label("Select a mesh primitive");
            }

            let can_assign = inspected_region.is_some() && selected_primitive.is_some();

            /*
             * Determine whether the selected primitive is currently
             * assigned to the inspected atlas region.
             */
            let is_assigned =
                if let (Some(region), Some(selection)) = (inspected_region, selected_primitive) {
                    region.has_mesh_assignment(
                        selection.mesh_id,
                        selection.object_index,
                        selection.primitive_index,
                    )
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
                        if let Some(selection) = selected_primitive {
                            if let Some(region_index) = editor_ui.inspected_atlas_region {
                                if let Some(region) = atlas
                                    .materials
                                    .iter_mut()
                                    .find(|region| region.index == region_index)
                                {
                                    region.unassign_mesh(
                                        selection.mesh_id,
                                        selection.object_index,
                                        selection.primitive_index,
                                    );

                                    atlas.uv_revision = atlas.uv_revision.wrapping_add(1);
                                }
                            }
                        }
                    }
                } else {
                    let response =
                        ui.add_enabled(can_assign, egui::Button::new("Assign to Atlas Region"));

                    if response.clicked() {
                        let Some(selection) = selected_primitive else {
                            return;
                        };

                        let Some(region_index) = editor_ui.inspected_atlas_region else {
                            return;
                        };

                        /*
                         * Remove this primitive from every atlas region
                         * before assigning it to the new region.
                         *
                         * This maintains the invariant that a primitive
                         * has at most one atlas assignment.
                         */
                        for region in &mut atlas.materials {
                            region.unassign_mesh(
                                selection.mesh_id,
                                selection.object_index,
                                selection.primitive_index,
                            );
                        }

                        /*
                         * Assign the primitive to the inspected atlas
                         * region.
                         */
                        if let Some(region) = atlas
                            .materials
                            .iter_mut()
                            .find(|region| region.index == region_index)
                        {
                            region.assign_mesh(
                                selection.mesh_id,
                                selection.object_index,
                                selection.primitive_index,
                            );

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

                    /*
                     * Remove assignments belonging to the deleted mesh.
                     *
                     * Do not modify AtlasMaterial::material_ref here.
                     * The source material/pipeline identity is independent
                     * from whether a mesh currently uses the atlas region.
                     */
                    for region in &mut atlas.materials {
                        region
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
