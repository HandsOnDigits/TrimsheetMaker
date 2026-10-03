use bevy_egui::egui;

use super::EditorUi;

use crate::{
    data::{Atlas, MaterialTexture},
    ui::{InspectorTab, panels::rendering::rendering_panel},
};

pub fn inspector_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.heading("Inspector");

    ui.horizontal(|ui| {
        if ui
            .selectable_label(
                editor_ui.inspector_tab == InspectorTab::Properties,
                "Properties",
            )
            .clicked()
        {
            editor_ui.inspector_tab = InspectorTab::Properties;
        }

        if ui
            .selectable_label(
                editor_ui.inspector_tab == InspectorTab::Rendering,
                "Rendering",
            )
            .clicked()
        {
            editor_ui.inspector_tab = InspectorTab::Rendering;
        }
    });

    ui.separator();

    match editor_ui.inspector_tab {
        InspectorTab::Properties => {
            properties_panel(ui, editor_ui, atlas);
        }

        InspectorTab::Rendering => {
            rendering_panel(ui, editor_ui, atlas);
        }
    }
}

fn properties_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    let Some(index) = editor_ui.inspected_atlas_region else {
        ui.vertical_centered(|ui| {
            ui.add_space(16.0);
            ui.label(egui::RichText::new("Nothing active").weak().size(15.0));
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Interact with an atlas region to inspect it.").weak());
        });

        return;
    };

    let mut transform_changed = false;

    {
        let Some(material) = atlas
            .materials
            .iter_mut()
            .find(|material| material.index == index)
        else {
            editor_ui.inspected_atlas_region = None;
            return;
        };

        ui.label(egui::RichText::new(format!("Atlas Region #{}", material.index)).strong());

        ui.add_space(8.0);

        egui::Grid::new("atlas_region_properties")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Name");

                ui.text_edit_singleline(&mut material.name);

                ui.end_row();

                ui.label("Position");

                ui.horizontal(|ui| {
                    ui.label("X");

                    if ui
                        .add(egui::DragValue::new(&mut material.transform.x).speed(1))
                        .changed()
                    {
                        transform_changed = true;
                    }

                    ui.label("Y");

                    if ui
                        .add(egui::DragValue::new(&mut material.transform.y).speed(1))
                        .changed()
                    {
                        transform_changed = true;
                    }
                });

                ui.end_row();

                ui.label("Size");

                ui.horizontal(|ui| {
                    ui.label("W");

                    if ui
                        .add(egui::DragValue::new(&mut material.transform.width).speed(1))
                        .changed()
                    {
                        transform_changed = true;
                    }

                    ui.label("H");

                    if ui
                        .add(egui::DragValue::new(&mut material.transform.height).speed(1))
                        .changed()
                    {
                        transform_changed = true;
                    }
                });

                ui.end_row();

                ui.label("Rotation");

                ui.horizontal(|ui| {
                    if ui.button("-90").clicked() {
                        material.transform.rotate_ccw();
                        transform_changed = true;
                    }

                    ui.label(format!("{}°", material.transform.rotation.degrees()));

                    if ui.button("+90").clicked() {
                        material.transform.rotate_cw();
                        transform_changed = true;
                    }
                });

                ui.end_row();
            });

        ui.add_space(16.0);
        ui.separator();

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Textures").strong());

            if ui.button("+").clicked() {
                material.textures.push(MaterialTexture {
                    id: String::new(),
                    path: std::path::PathBuf::new(),
                    image: None,
                    egui_texture: None,
                });
            }
        });

        ui.add_space(8.0);

        let mut remove_index = None;

        for (i, texture) in material.textures.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let display_name = editor_ui
                        .project_settings
                        .texture_naming
                        .groups
                        .iter()
                        .find(|group| group.id == texture.id)
                        .map(|group| group.name.as_str())
                        .unwrap_or("Unclassified");

                    ui.label(egui::RichText::new(display_name).strong());

                    if ui.small_button("×").clicked() {
                        remove_index = Some(i);
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

        if let Some(index) = remove_index {
            material.textures.remove(index);
        }
    }

    if transform_changed {
        atlas.mark_dirty();
    }
}
