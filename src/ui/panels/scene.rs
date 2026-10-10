use bevy::prelude::*;

use bevy_egui::egui;

use crate::{
    data::Atlas,
    history::{EditorHistory, EditorSnapshot},
    hotkeys::{EditorAction, HotkeyManager, execute_action},
    mesh::{MeshLight, MeshSceneState},
    ui::EditorUi,
    viewport::{MeshPreview, MeshPrimitiveSelection},
};

pub fn scene_panel(
    ui: &mut egui::Ui,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    images: &mut Assets<Image>,
    hotkeys: &HotkeyManager,
    mesh_preview: &mut MeshPreview,
    mesh_light: &mut MeshLight,
    mesh_scene_state: &mut MeshSceneState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    history: &mut EditorHistory,
) {
    let before = EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state);

    let visibility_before: Vec<(u64, bool, Vec<bool>)> = mesh_scene_state
        .meshes
        .iter()
        .map(|mesh| {
            (
                mesh.id,
                mesh.visible,
                mesh.parts.iter().map(|part| part.visible).collect(),
            )
        })
        .collect();

    ui.heading("Scene");
    ui.label(format!("{} meshes", mesh_scene_state.meshes.len()));
    ui.separator();

    if mesh_scene_state.meshes.is_empty() {
        ui.label("No meshes imported.");
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("scene_hierarchy_scroll")
        .show(ui, |ui| {
            for mesh in &mut mesh_scene_state.meshes {
                let mesh_id = mesh.id;
                let mesh_selected = editor_ui.selected_mesh == Some(mesh_id);
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
                        mesh.name.as_str()
                    };

                    if ui.selectable_label(mesh_selected, name).clicked() {
                        editor_ui.selected_mesh = Some(mesh_id);
                        editor_ui.selected_mesh_object = None;
                        mesh_preview.selected_mesh_id = Some(mesh_id);
                        mesh_preview.selected_primitive = None;
                    }
                });

                if !expanded {
                    continue;
                }

                ui.indent(("scene_mesh_children", mesh_id), |ui| {
                    for (object_index, object) in mesh.scene.objects.iter().enumerate() {
                        let object_key = (mesh_id, object_index);
                        let object_expanded = editor_ui.expanded_mesh_objects.contains(&object_key);
                        let object_selected = editor_ui.selected_mesh_object == Some(object_key);

                        let all_visible = mesh
                            .parts
                            .iter()
                            .filter(|part| part.object_index == object_index)
                            .all(|part| part.visible);

                        let any_visible = mesh
                            .parts
                            .iter()
                            .any(|part| part.object_index == object_index && part.visible);

                        let object_name = if object.name.is_empty() {
                            format!("Object {}", object_index + 1)
                        } else {
                            object.name.clone()
                        };

                        ui.horizontal(|ui| {
                            let mut object_visible = all_visible;
                            let checkbox = ui.add_enabled(
                                mesh.parts
                                    .iter()
                                    .any(|part| part.object_index == object_index),
                                egui::Checkbox::new(&mut object_visible, ""),
                            );

                            if checkbox.clicked() {
                                let target_visible = !all_visible;

                                for part in mesh
                                    .parts
                                    .iter_mut()
                                    .filter(|part| part.object_index == object_index)
                                {
                                    part.visible = target_visible;
                                }
                            }

                            if any_visible && !all_visible {
                                let rect = checkbox.rect;
                                let center = rect.center();
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(center.x - 4.0, center.y),
                                        egui::pos2(center.x + 4.0, center.y),
                                    ],
                                    egui::Stroke::new(2.0, ui.visuals().text_color()),
                                );
                            }

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
                                    editor_ui.expanded_mesh_objects.remove(&object_key);
                                } else {
                                    editor_ui.expanded_mesh_objects.insert(object_key);
                                }
                            }

                            if ui
                                .selectable_label(
                                    object_selected,
                                    egui::RichText::new(object_name).strong(),
                                )
                                .clicked()
                            {
                                editor_ui.selected_mesh = Some(mesh_id);
                                editor_ui.selected_mesh_object = Some(object_key);
                                mesh_preview.selected_mesh_id = Some(mesh_id);
                                mesh_preview.selected_primitive = None;
                            }
                        });

                        if !object_expanded {
                            continue;
                        }

                        ui.indent(("scene_object_parts", mesh_id, object_index), |ui| {
                            for part in mesh
                                .parts
                                .iter_mut()
                                .filter(|part| part.object_index == object_index)
                            {
                                let selection = MeshPrimitiveSelection {
                                    mesh_id,
                                    object_index: part.object_index,
                                    primitive_index: part.primitive_index,
                                };

                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut part.visible, "");

                                    let selected =
                                        mesh_preview.selected_primitive == Some(selection);

                                    if ui
                                        .selectable_label(
                                            selected,
                                            format!("Primitive {}", part.primitive_index + 1),
                                        )
                                        .clicked()
                                    {
                                        editor_ui.selected_mesh = Some(mesh_id);
                                        editor_ui.selected_mesh_object = None;
                                        mesh_preview.selected_mesh_id = Some(mesh_id);
                                        mesh_preview.selected_primitive = Some(selection);
                                    }
                                });
                            }
                        });
                    }
                });
            }

            ui.separator();
            ui.heading("Actions");

            let has_primitive = mesh_preview.selected_primitive.is_some();
            let has_object = editor_ui.selected_mesh_object.is_some();
            let has_mesh = editor_ui.selected_mesh.is_some();

            let has_assignment_target = editor_ui.inspected_atlas_region.is_some();

            ui.add_enabled_ui(has_primitive || has_object, |ui| {
                ui.add_enabled_ui(has_assignment_target, |ui| {
                    if hotkeys.menu_button(
                        ui,
                        EditorAction::AssignMaterial,
                        "Assign to Inspected Region",
                    ) {
                        execute_action(
                            EditorAction::AssignMaterial,
                            editor_ui,
                            atlas,
                            images,
                            mesh_preview,
                            mesh_light,
                            mesh_scene_state,
                            commands,
                            meshes,
                            history,
                        );
                    }
                });

                if hotkeys.menu_button(ui, EditorAction::UnassignMaterial, "Unassign Material") {
                    execute_action(
                        EditorAction::UnassignMaterial,
                        editor_ui,
                        atlas,
                        images,
                        mesh_preview,
                        mesh_light,
                        mesh_scene_state,
                        commands,
                        meshes,
                        history,
                    );
                }
            });

            ui.separator();

            if has_primitive {
                ui.add_enabled_ui(has_primitive || has_object, |ui| {
                    ui.add_enabled_ui(has_assignment_target, |ui| {
                        if hotkeys.menu_button(
                            ui,
                            EditorAction::AssignMaterial,
                            "Assign to Inspected Region",
                        ) {
                            execute_action(
                                EditorAction::AssignMaterial,
                                editor_ui,
                                atlas,
                                images,
                                mesh_preview,
                                mesh_light,
                                mesh_scene_state,
                                commands,
                                meshes,
                                history,
                            );
                        }
                    });

                    if hotkeys.menu_button(ui, EditorAction::UnassignMaterial, "Unassign Material")
                    {
                        execute_action(
                            EditorAction::UnassignMaterial,
                            editor_ui,
                            atlas,
                            images,
                            mesh_preview,
                            mesh_light,
                            mesh_scene_state,
                            commands,
                            meshes,
                            history,
                        );
                    }
                });

                ui.separator();

                if has_primitive {
                    if hotkeys.menu_button(
                        ui,
                        EditorAction::DeleteMeshPrimitive,
                        "Delete Primitive",
                    ) {
                        execute_action(
                            EditorAction::DeleteMeshPrimitive,
                            editor_ui,
                            atlas,
                            images,
                            mesh_preview,
                            mesh_light,
                            mesh_scene_state,
                            commands,
                            meshes,
                            history,
                        );
                    }
                } else if has_object {
                    if hotkeys.menu_button(ui, EditorAction::DeleteMeshObject, "Delete Object") {
                        execute_action(
                            EditorAction::DeleteMeshObject,
                            editor_ui,
                            atlas,
                            images,
                            mesh_preview,
                            mesh_light,
                            mesh_scene_state,
                            commands,
                            meshes,
                            history,
                        );
                    }
                } else if has_mesh {
                    if hotkeys.menu_button(ui, EditorAction::DeleteMesh, "Delete Mesh") {
                        execute_action(
                            EditorAction::DeleteMesh,
                            editor_ui,
                            atlas,
                            images,
                            mesh_preview,
                            mesh_light,
                            mesh_scene_state,
                            commands,
                            meshes,
                            history,
                        );
                    }
                }
            }
        });

    let visibility_changed = mesh_scene_state.meshes.iter().any(|mesh| {
        visibility_before
            .iter()
            .find(|(id, _, _)| *id == mesh.id)
            .is_some_and(|(_, was_visible, parts_before)| {
                *was_visible != mesh.visible
                    || parts_before.len() != mesh.parts.len()
                    || parts_before
                        .iter()
                        .zip(&mesh.parts)
                        .any(|(was_visible, part)| *was_visible != part.visible)
            })
    });

    if visibility_changed {
        history.push(before);
    }
}
