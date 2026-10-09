use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    data::Atlas,
    history::{EditorHistory, EditorSnapshot},
    hotkeys::{EditorAction, EditorActionMessage},
    mesh::MeshSceneState,
    ui::EditorUi,
    viewport::{MeshPreview, MeshPrimitiveSelection},
};

use super::FloatingWindowInteraction;

pub fn meshes_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    mesh_scene_state: &mut MeshSceneState,
    mesh_preview: &mut MeshPreview,
    action_messages: &mut MessageWriter<EditorActionMessage>,
    history: &mut EditorHistory,
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
                editor_ui.clear_mesh_selection();
                mesh_preview.clear_selection();

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
                            editor_ui.selected_mesh_object = None;

                            mesh_preview.selected_mesh_id = Some(mesh_id);
                            mesh_preview.selected_primitive = None;
                        }
                    });

                    if expanded {
                        ui.indent(egui::Id::new(("mesh_objects", mesh_id)), |ui| {
                            egui::Frame::NONE.show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .max_height(250.0)
                                    .auto_shrink([false, true])
                                    .show(ui, |ui| {
                                        ui.spacing_mut().item_spacing.y = 2.0;

                                        for (object_index, object) in
                                            mesh.scene.objects.iter().enumerate()
                                        {
                                            let object_key = (mesh_id, object_index);

                                            let object_name = if object.name.is_empty() {
                                                format!("Object {}", object_index + 1)
                                            } else {
                                                object.name.clone()
                                            };

                                            let object_selected =
                                                editor_ui.selected_mesh_object == Some(object_key);
                                            let object_expanded = editor_ui
                                                .expanded_mesh_objects
                                                .contains(&object_key);

                                            ui.horizontal(|ui| {
                                                let arrow = if object_expanded { "v" } else { ">" };

                                                if ui
                                                    .add(
                                                        egui::Button::new(arrow)
                                                            .frame(false)
                                                            .min_size(egui::vec2(18.0, 18.0)),
                                                    )
                                                    .clicked()
                                                {
                                                    if object_expanded {
                                                        editor_ui
                                                            .expanded_mesh_objects
                                                            .remove(&object_key);
                                                    } else {
                                                        editor_ui
                                                            .expanded_mesh_objects
                                                            .insert(object_key);
                                                    }
                                                }

                                                if ui
                                                    .selectable_label(
                                                        object_selected,
                                                        egui::RichText::new(object_name)
                                                            .strong()
                                                            .small(),
                                                    )
                                                    .clicked()
                                                {
                                                    editor_ui.selected_mesh = Some(mesh_id);
                                                    editor_ui.selected_mesh_object =
                                                        Some(object_key);

                                                    mesh_preview.selected_mesh_id = Some(mesh_id);
                                                    mesh_preview.selected_primitive = None;
                                                }
                                            });

                                            if object_expanded {
                                                ui.indent(
                                                    egui::Id::new((
                                                        "mesh_object_parts",
                                                        mesh_id,
                                                        object_index,
                                                    )),
                                                    |ui| {
                                                        for part in
                                                            mesh.parts.iter_mut().filter(|part| {
                                                                part.object_index == object_index
                                                            })
                                                        {
                                                            let selected = mesh_preview
                                                                .selected_primitive
                                                                == Some(MeshPrimitiveSelection {
                                                                    mesh_id,
                                                                    object_index: part.object_index,
                                                                    primitive_index: part
                                                                        .primitive_index,
                                                                });

                                                            ui.horizontal(|ui| {
                                                                ui.add_space(4.0);

                                                                ui.checkbox(&mut part.visible, "");

                                                                if ui
                                                                    .selectable_label(
                                                                        selected,
                                                                        egui::RichText::new(
                                                                            part.name.clone(),
                                                                        )
                                                                        .small(),
                                                                    )
                                                                    .clicked()
                                                                {
                                                                    editor_ui.selected_mesh =
                                                                        Some(mesh_id);
                                                                    editor_ui
                                                                        .selected_mesh_object =
                                                                        None;

                                                                    mesh_preview.selected_mesh_id =
                                                                        Some(mesh_id);
                                                                    mesh_preview
                                                                        .selected_primitive = Some(
                                                                        MeshPrimitiveSelection {
                                                                            mesh_id,
                                                                            object_index: part
                                                                                .object_index,
                                                                            primitive_index: part
                                                                                .primitive_index,
                                                                        },
                                                                    );
                                                                }
                                                            });
                                                        }
                                                    },
                                                );
                                            }
                                        }
                                    });
                            });
                        });
                    }
                }
            });

            ui.separator();

            // Atlas assignment
            let selected_primitives: Vec<MeshPrimitiveSelection> =
                if let Some(selection) = mesh_preview.selected_primitive {
                    vec![selection]
                } else if let Some((mesh_id, object_index)) = editor_ui.selected_mesh_object {
                    mesh_scene_state
                        .meshes
                        .iter()
                        .find(|mesh| mesh.id == mesh_id)
                        .map(|mesh| {
                            mesh.parts
                                .iter()
                                .filter(|part| part.object_index == object_index)
                                .map(|part| MeshPrimitiveSelection {
                                    mesh_id,
                                    object_index: part.object_index,
                                    primitive_index: part.primitive_index,
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

            let has_selection = !selected_primitives.is_empty();

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

            if !has_selection {
                ui.label("Select a mesh object or primitive");
            }

            let can_assign = inspected_region.is_some() && has_selection;

            let is_assigned = inspected_region.is_some()
                && has_selection
                && selected_primitives.iter().all(|selection| {
                    inspected_region.unwrap().has_mesh_assignment(
                        selection.mesh_id,
                        selection.object_index,
                        selection.primitive_index,
                    )
                });

            ui.horizontal(|ui| {
                if is_assigned {
                    ui.label(
                        egui::RichText::new("Assigned")
                            .strong()
                            .color(egui::Color32::LIGHT_GREEN),
                    );

                    if ui.button("Unassign").clicked() {
                        let before = EditorSnapshot::capture(
                            atlas,
                            &editor_ui.project_settings,
                            mesh_scene_state,
                        );

                        for selection in &selected_primitives {
                            for region in &mut atlas.materials {
                                region.unassign_mesh(
                                    selection.mesh_id,
                                    selection.object_index,
                                    selection.primitive_index,
                                );
                            }
                        }

                        atlas.uv_revision = atlas.uv_revision.wrapping_add(1);
                        history.push(before);
                    }
                } else {
                    let response =
                        ui.add_enabled(can_assign, egui::Button::new("Assign to Atlas Region"));

                    if response.clicked() {
                        assign_selected_primitives(
                            editor_ui,
                            atlas,
                            mesh_scene_state,
                            mesh_preview,
                            history,
                        );
                    }
                }
            });

            ui.separator();

            let has_selection = mesh_preview.selected_primitive.is_some()
                || editor_ui.selected_mesh_object.is_some();

            if ui
                .add_enabled(has_selection, egui::Button::new("Delete Selected"))
                .clicked()
            {
                action_messages.write(EditorActionMessage::new(EditorAction::RemoveSelectedMesh));
            }
        });

    let owns_pointer = window_response
        .as_ref()
        .map(|response| response.owns_pointer(ctx))
        .unwrap_or(false);

    editor_ui.show_meshes_window = open;

    owns_pointer
}

pub fn assign_selected_primitives(
    editor_ui: &EditorUi,
    atlas: &mut Atlas,
    mesh_scene_state: &MeshSceneState,
    mesh_preview: &MeshPreview,
    history: &mut EditorHistory,
) {
    let selected_primitives: Vec<MeshPrimitiveSelection> =
        if let Some(selection) = mesh_preview.selected_primitive {
            vec![selection]
        } else if let Some((mesh_id, object_index)) = editor_ui.selected_mesh_object {
            mesh_scene_state
                .meshes
                .iter()
                .find(|mesh| mesh.id == mesh_id)
                .map(|mesh| {
                    mesh.parts
                        .iter()
                        .filter(|part| part.object_index == object_index)
                        .map(|part| MeshPrimitiveSelection {
                            mesh_id,
                            object_index,
                            primitive_index: part.primitive_index,
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

    let Some(region_index) = editor_ui.inspected_atlas_region else {
        return;
    };

    if selected_primitives.is_empty() {
        return;
    }

    let Some(region_exists) = atlas
        .materials
        .iter()
        .any(|region| region.index == region_index)
        .then_some(())
    else {
        return;
    };

    let _ = region_exists;

    let before = EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

    for selection in &selected_primitives {
        for region in &mut atlas.materials {
            region.unassign_mesh(
                selection.mesh_id,
                selection.object_index,
                selection.primitive_index,
            );
        }
    }

    if let Some(region) = atlas
        .materials
        .iter_mut()
        .find(|region| region.index == region_index)
    {
        for selection in &selected_primitives {
            region.assign_mesh(
                selection.mesh_id,
                selection.object_index,
                selection.primitive_index,
            );
        }

        atlas.uv_revision = atlas.uv_revision.wrapping_add(1);
        history.push(before);
    }
}

pub fn unassign_selected_primitives(
    editor_ui: &EditorUi,
    atlas: &mut Atlas,
    mesh_scene_state: &MeshSceneState,
    mesh_preview: &MeshPreview,
    history: &mut EditorHistory,
) {
    let selected_primitives: Vec<MeshPrimitiveSelection> =
        if let Some(selection) = mesh_preview.selected_primitive {
            vec![selection]
        } else if let Some((mesh_id, object_index)) = editor_ui.selected_mesh_object {
            mesh_scene_state
                .meshes
                .iter()
                .find(|mesh| mesh.id == mesh_id)
                .map(|mesh| {
                    mesh.parts
                        .iter()
                        .filter(|part| part.object_index == object_index)
                        .map(|part| MeshPrimitiveSelection {
                            mesh_id,
                            object_index,
                            primitive_index: part.primitive_index,
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

    if selected_primitives.is_empty() {
        return;
    }

    let has_assignments = selected_primitives.iter().any(|selection| {
        atlas.materials.iter().any(|region| {
            region.has_mesh_assignment(
                selection.mesh_id,
                selection.object_index,
                selection.primitive_index,
            )
        })
    });

    if !has_assignments {
        return;
    }

    let before = EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

    for selection in &selected_primitives {
        for region in &mut atlas.materials {
            region.unassign_mesh(
                selection.mesh_id,
                selection.object_index,
                selection.primitive_index,
            );
        }
    }

    atlas.uv_revision = atlas.uv_revision.wrapping_add(1);
    history.push(before);
}
