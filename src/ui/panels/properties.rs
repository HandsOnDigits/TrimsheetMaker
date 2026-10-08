use bevy_egui::egui;

use crate::{
    data::Atlas,
    history::{EditorHistory, EditorSnapshot},
    mesh::MeshSceneState,
};

use super::EditorUi;

pub fn properties_panel(
    ui: &mut egui::Ui,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    history: &mut EditorHistory,
    mesh_scene_state: &mut MeshSceneState,
) {
    let Some(index) = editor_ui.inspected_atlas_region else {
        ui.vertical_centered(|ui| {
            ui.add_space(16.0);
            ui.label(egui::RichText::new("Nothing active").weak().size(15.0));
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Interact with an atlas region to inspect it.").weak());
        });

        return;
    };

    if !atlas
        .materials
        .iter()
        .any(|material| material.index == index)
    {
        editor_ui.inspected_atlas_region = None;
        editor_ui.properties_snapshot = None;
        return;
    }

    let capture_snapshot = |atlas: &Atlas,
                            editor_ui: &EditorUi,
                            mesh_scene_state: &MeshSceneState|
     -> EditorSnapshot {
        EditorSnapshot::capture(atlas, &editor_ui.project_settings, mesh_scene_state)
    };

    /*
     * NAME
     *
     * Start a transaction when the field receives focus and commit it
     * when focus is lost.
     */
    {
        let Some(material) = atlas
            .materials
            .iter()
            .find(|material| material.index == index)
        else {
            return;
        };

        ui.label(egui::RichText::new(format!("Atlas Region #{}", material.index)).strong());
    }

    ui.add_space(8.0);

    /*
     * Name
     */
    {
        let current_name = atlas
            .materials
            .iter()
            .find(|material| material.index == index)
            .map(|material| material.name.clone())
            .unwrap_or_default();

        let mut name = current_name;

        let response = ui.text_edit_singleline(&mut name);

        if response.gained_focus() {
            editor_ui.properties_snapshot =
                Some(capture_snapshot(atlas, editor_ui, mesh_scene_state));
        }

        if response.changed() {
            if let Some(material) = atlas
                .materials
                .iter_mut()
                .find(|material| material.index == index)
            {
                material.name = name;
            }
        }

        if response.lost_focus() {
            if let Some(snapshot) = editor_ui.properties_snapshot.take() {
                history.push(snapshot);
            }
        }
    }

    ui.add_space(8.0);

    let mut transform_changed = false;

    /*
     * POSITION
     */
    ui.label("Position");

    ui.horizontal(|ui| {
        ui.label("X");

        let x = atlas
            .materials
            .iter()
            .find(|material| material.index == index)
            .map(|material| material.transform.x)
            .unwrap_or_default();

        let mut value = x;

        let response = ui.add(egui::DragValue::new(&mut value).speed(1));

        if response.drag_started() {
            editor_ui.properties_snapshot =
                Some(capture_snapshot(atlas, editor_ui, mesh_scene_state));
        }

        if response.changed() {
            if let Some(material) = atlas
                .materials
                .iter_mut()
                .find(|material| material.index == index)
            {
                material.transform.x = value;
                transform_changed = true;
            }
        }

        if response.drag_stopped() {
            if let Some(snapshot) = editor_ui.properties_snapshot.take() {
                history.push(snapshot);
            }
        }

        ui.label("Y");

        let y = atlas
            .materials
            .iter()
            .find(|material| material.index == index)
            .map(|material| material.transform.y)
            .unwrap_or_default();

        let mut value = y;

        let response = ui.add(egui::DragValue::new(&mut value).speed(1));

        if response.drag_started() {
            editor_ui.properties_snapshot =
                Some(capture_snapshot(atlas, editor_ui, mesh_scene_state));
        }

        if response.changed() {
            if let Some(material) = atlas
                .materials
                .iter_mut()
                .find(|material| material.index == index)
            {
                material.transform.y = value;
                transform_changed = true;
            }
        }

        if response.drag_stopped() {
            if let Some(snapshot) = editor_ui.properties_snapshot.take() {
                history.push(snapshot);
            }
        }
    });

    ui.end_row();

    /*
     * SIZE
     */
    ui.label("Size");

    ui.horizontal(|ui| {
        ui.label("W");

        let width = atlas
            .materials
            .iter()
            .find(|material| material.index == index)
            .map(|material| material.transform.width)
            .unwrap_or_default();

        let mut value = width;

        let response = ui.add(egui::DragValue::new(&mut value).speed(1));

        if response.drag_started() {
            editor_ui.properties_snapshot =
                Some(capture_snapshot(atlas, editor_ui, mesh_scene_state));
        }

        if response.changed() {
            if let Some(material) = atlas
                .materials
                .iter_mut()
                .find(|material| material.index == index)
            {
                material.transform.width = value;
                transform_changed = true;
            }
        }

        if response.drag_stopped() {
            if let Some(snapshot) = editor_ui.properties_snapshot.take() {
                history.push(snapshot);
            }
        }

        ui.label("H");

        let height = atlas
            .materials
            .iter()
            .find(|material| material.index == index)
            .map(|material| material.transform.height)
            .unwrap_or_default();

        let mut value = height;

        let response = ui.add(egui::DragValue::new(&mut value).speed(1));

        if response.drag_started() {
            editor_ui.properties_snapshot =
                Some(capture_snapshot(atlas, editor_ui, mesh_scene_state));
        }

        if response.changed() {
            if let Some(material) = atlas
                .materials
                .iter_mut()
                .find(|material| material.index == index)
            {
                material.transform.height = value;
                transform_changed = true;
            }
        }

        if response.drag_stopped() {
            if let Some(snapshot) = editor_ui.properties_snapshot.take() {
                history.push(snapshot);
            }
        }
    });

    ui.end_row();

    /*
     * ROTATION
     */
    ui.label("Rotation");

    ui.horizontal(|ui| {
        if ui.button("-90").clicked() {
            let snapshot = capture_snapshot(atlas, editor_ui, mesh_scene_state);

            if let Some(material) = atlas
                .materials
                .iter_mut()
                .find(|material| material.index == index)
            {
                material.transform.rotate_ccw();
                transform_changed = true;
            }

            history.push(snapshot);
        }

        let rotation = atlas
            .materials
            .iter()
            .find(|material| material.index == index)
            .map(|material| material.transform.rotation.degrees())
            .unwrap_or_default();

        ui.label(format!("{rotation}°"));

        if ui.button("+90").clicked() {
            let snapshot = capture_snapshot(atlas, editor_ui, mesh_scene_state);

            if let Some(material) = atlas
                .materials
                .iter_mut()
                .find(|material| material.index == index)
            {
                material.transform.rotate_cw();
                transform_changed = true;
            }

            history.push(snapshot);
        }
    });

    ui.add_space(16.0);
    ui.separator();

    /*
     * UV TRANSFORM
     */
    let mesh_assignments = atlas
        .materials
        .iter()
        .find(|material| material.index == index)
        .map(|material| material.mesh_assignments.clone())
        .unwrap_or_default();

    if !mesh_assignments.is_empty() {
        ui.label(egui::RichText::new("UV Transform").strong());

        let first_transform = mesh_assignments.iter().find_map(|assignment| {
            mesh_scene_state
                .get(assignment.mesh_id)
                .and_then(|mesh| mesh.scene.objects.get(assignment.object_index))
                .and_then(|object| object.primitives.get(assignment.primitive_index))
                .map(|primitive| primitive.uv_transform)
        });

        if let Some(uv_transform) = first_transform {
            let mut uv_changed = false;

            /*
             * POSITION
             */
            ui.label("Position");

            ui.horizontal(|ui| {
                ui.label("X");

                let mut value = uv_transform.translation.x;

                let response = ui.add(egui::DragValue::new(&mut value).speed(0.01));

                if response.drag_started() {
                    editor_ui.properties_snapshot =
                        Some(capture_snapshot(atlas, editor_ui, mesh_scene_state));
                }

                if response.changed() {
                    for assignment in &mesh_assignments {
                        let Some(mesh) = mesh_scene_state.get_mut(assignment.mesh_id) else {
                            continue;
                        };

                        let Some(object) = mesh.scene.objects.get_mut(assignment.object_index)
                        else {
                            continue;
                        };

                        let Some(primitive) = object.primitives.get_mut(assignment.primitive_index)
                        else {
                            continue;
                        };

                        primitive.uv_transform.translation.x = value;
                    }

                    uv_changed = true;
                }

                if response.drag_stopped() {
                    if let Some(snapshot) = editor_ui.properties_snapshot.take() {
                        history.push(snapshot);
                    }
                }

                ui.label("Y");

                let mut value = uv_transform.translation.y;

                let response = ui.add(egui::DragValue::new(&mut value).speed(0.01));

                if response.drag_started() {
                    editor_ui.properties_snapshot =
                        Some(capture_snapshot(atlas, editor_ui, mesh_scene_state));
                }

                if response.changed() {
                    for assignment in &mesh_assignments {
                        let Some(mesh) = mesh_scene_state.get_mut(assignment.mesh_id) else {
                            continue;
                        };

                        let Some(object) = mesh.scene.objects.get_mut(assignment.object_index)
                        else {
                            continue;
                        };

                        let Some(primitive) = object.primitives.get_mut(assignment.primitive_index)
                        else {
                            continue;
                        };

                        primitive.uv_transform.translation.y = value;
                    }

                    uv_changed = true;
                }

                if response.drag_stopped() {
                    if let Some(snapshot) = editor_ui.properties_snapshot.take() {
                        history.push(snapshot);
                    }
                }
            });

            /*
             * SCALE
             */
            ui.label("Scale");

            ui.horizontal(|ui| {
                ui.label("X");

                let mut value = uv_transform.scale.x;

                let response = ui.add(egui::DragValue::new(&mut value).speed(0.01));

                if response.drag_started() {
                    editor_ui.properties_snapshot =
                        Some(capture_snapshot(atlas, editor_ui, mesh_scene_state));
                }

                if response.changed() {
                    for assignment in &mesh_assignments {
                        let Some(mesh) = mesh_scene_state.get_mut(assignment.mesh_id) else {
                            continue;
                        };

                        let Some(object) = mesh.scene.objects.get_mut(assignment.object_index)
                        else {
                            continue;
                        };

                        let Some(primitive) = object.primitives.get_mut(assignment.primitive_index)
                        else {
                            continue;
                        };

                        primitive.uv_transform.scale.x = value;
                    }

                    uv_changed = true;
                }

                if response.drag_stopped() {
                    if let Some(snapshot) = editor_ui.properties_snapshot.take() {
                        history.push(snapshot);
                    }
                }

                ui.label("Y");

                let mut value = uv_transform.scale.y;

                let response = ui.add(egui::DragValue::new(&mut value).speed(0.01));

                if response.drag_started() {
                    editor_ui.properties_snapshot =
                        Some(capture_snapshot(atlas, editor_ui, mesh_scene_state));
                }

                if response.changed() {
                    for assignment in &mesh_assignments {
                        let Some(mesh) = mesh_scene_state.get_mut(assignment.mesh_id) else {
                            continue;
                        };

                        let Some(object) = mesh.scene.objects.get_mut(assignment.object_index)
                        else {
                            continue;
                        };

                        let Some(primitive) = object.primitives.get_mut(assignment.primitive_index)
                        else {
                            continue;
                        };

                        primitive.uv_transform.scale.y = value;
                    }

                    uv_changed = true;
                }

                if response.drag_stopped() {
                    if let Some(snapshot) = editor_ui.properties_snapshot.take() {
                        history.push(snapshot);
                    }
                }
            });

            /*
             * ROTATION
             */
            ui.label("Rotation");

            ui.horizontal(|ui| {
                let mut value = uv_transform.rotation.to_degrees();

                let response = ui.add(egui::DragValue::new(&mut value).speed(1.0).suffix("°"));

                if response.drag_started() {
                    editor_ui.properties_snapshot =
                        Some(capture_snapshot(atlas, editor_ui, mesh_scene_state));
                }

                if response.changed() {
                    let rotation = value.to_radians();

                    for assignment in &mesh_assignments {
                        let Some(mesh) = mesh_scene_state.get_mut(assignment.mesh_id) else {
                            continue;
                        };

                        let Some(object) = mesh.scene.objects.get_mut(assignment.object_index)
                        else {
                            continue;
                        };

                        let Some(primitive) = object.primitives.get_mut(assignment.primitive_index)
                        else {
                            continue;
                        };

                        primitive.uv_transform.rotation = rotation;
                    }

                    uv_changed = true;
                }

                if response.drag_stopped() {
                    if let Some(snapshot) = editor_ui.properties_snapshot.take() {
                        history.push(snapshot);
                    }
                }
            });

            if uv_changed {
                mesh_scene_state.mark_uv_dirty();
            }

            ui.add_space(16.0);
            ui.separator();
        }
    }
}
