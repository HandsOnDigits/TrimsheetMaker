use bevy_egui::egui;

use super::EditorUi;

use crate::data::{Atlas, MaterialTexture};

pub fn inspector_panel(ui: &mut egui::Ui, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    ui.heading("Inspector");

    ui.separator();

    let Some(index) = editor_ui.selected_material else {
        ui.vertical_centered(|ui| {
            ui.add_space(16.0);

            ui.label(egui::RichText::new("Nothing selected").weak().size(15.0));

            ui.add_space(4.0);

            ui.label(egui::RichText::new("Select a material from the list.").weak());
        });

        return;
    };

    let Some(material) = atlas.selected_mut(Some(index)) else {
        editor_ui.selected_material = None;
        return;
    };

    ui.label(egui::RichText::new(format!("Material #{}", material.index)).strong());

    ui.add_space(8.0);

    egui::Grid::new("material_properties")
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut material.name);
            ui.end_row();

            ui.label("Position");

            ui.horizontal(|ui| {
                ui.label("X");
                ui.add(egui::DragValue::new(&mut material.x).speed(1));

                ui.label("Y");
                ui.add(egui::DragValue::new(&mut material.y).speed(1));
            });

            ui.end_row();

            ui.label("Size");

            ui.horizontal(|ui| {
                ui.label("W");
                ui.add(egui::DragValue::new(&mut material.width).speed(1));

                ui.label("H");
                ui.add(egui::DragValue::new(&mut material.height).speed(1));
            });

            ui.end_row();
        });

    ui.add_space(16.0);

    ui.separator();

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Textures").strong());

        if ui.button("+").clicked() {
            material.textures.push(MaterialTexture {
                group: "Texture".into(),
                path: std::path::PathBuf::new(),
                image: None,
            });
        }
    });

    ui.add_space(8.0);

    let mut remove_index = None;

    for (i, texture) in material.textures.iter_mut().enumerate() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&texture.group).strong());

                if ui.small_button("×").clicked() {
                    remove_index = Some(i);
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("File");

                let path = texture.path.display().to_string();

                ui.add(
                    egui::TextEdit::singleline(&mut path.clone())
                        .desired_width(f32::INFINITY)
                        .interactive(false),
                );
            });
        });

        ui.add_space(6.0);
    }

    if let Some(index) = remove_index {
        material.textures.remove(index);
    }
}
