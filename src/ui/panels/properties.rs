use std::path::PathBuf;

use bevy_egui::egui;

use crate::{
    data::{Atlas, MaterialTexture},
    history::{EditorHistory, EditorSnapshot},
    mesh::MeshSceneState,
};

use super::EditorUi;

pub fn properties_panel(
    ui: &mut egui::Ui,
    editor_ui: &mut EditorUi,
    atlas: &mut Atlas,
    history: &mut EditorHistory,
    mesh_scene_state: &MeshSceneState,
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
     * TEXTURES
     */
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Textures").strong());

        if ui.button("+").clicked() {
            let snapshot = capture_snapshot(atlas, editor_ui, mesh_scene_state);

            if let Some(material) = atlas
                .materials
                .iter_mut()
                .find(|material| material.index == index)
            {
                material.textures.push(MaterialTexture {
                    id: String::new(),
                    path: PathBuf::new(),
                    group_id: None,
                    image: None,
                    egui_texture: None,
                });
            }

            history.push(snapshot);
        }
    });

    ui.add_space(8.0);

    let mut remove_texture_index = None;

    /*
     * Render texture list.
     */
    {
        let Some(material) = atlas
            .materials
            .iter()
            .find(|material| material.index == index)
        else {
            return;
        };

        for (i, texture) in material.textures.iter().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let display_name = texture
                        .group_id
                        .as_ref()
                        .and_then(|group_id| {
                            editor_ui
                                .project_settings
                                .texture_naming
                                .groups
                                .iter()
                                .find(|group| &group.id == group_id)
                        })
                        .map(|group| group.name.as_str())
                        .unwrap_or("Unclassified");

                    ui.label(egui::RichText::new(display_name).strong());

                    if ui.small_button("×").clicked() {
                        remove_texture_index = Some(i);
                    }
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("File");

                    let path = texture.path.display().to_string();

                    ui.add(egui::TextEdit::singleline(&mut path.clone()).interactive(false));
                });
            });

            ui.add_space(6.0);
        }
    }

    /*
     * Remove texture after the immutable rendering pass.
     */
    if let Some(texture_index) = remove_texture_index {
        let texture_exists = atlas
            .materials
            .iter()
            .find(|material| material.index == index)
            .map(|material| texture_index < material.textures.len())
            .unwrap_or(false);

        if texture_exists {
            let snapshot = capture_snapshot(atlas, editor_ui, mesh_scene_state);

            if let Some(material) = atlas
                .materials
                .iter_mut()
                .find(|material| material.index == index)
            {
                material.textures.remove(texture_index);
            }

            history.push(snapshot);
        }
    }

    if transform_changed {
        atlas.mark_dirty();
    }
}
