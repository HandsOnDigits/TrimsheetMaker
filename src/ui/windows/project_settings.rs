use bevy_egui::egui;

use crate::{data::Atlas, ui::EditorUi};

pub fn project_settings_window(ctx: &egui::Context, editor_ui: &mut EditorUi, atlas: &mut Atlas) {
    let mut open = editor_ui.show_project_settings_window;

    egui::Window::new("Project Settings")
        .open(&mut open)
        .resizable(true)
        .default_width(560.0)
        .show(ctx, |ui| {
            ui.heading("Atlas");

            ui.horizontal(|ui| {
                ui.label("Width");

                if ui
                    .add(
                        egui::DragValue::new(&mut editor_ui.project_settings.atlas_width)
                            .range(1..=16384),
                    )
                    .changed()
                {
                    atlas.width = editor_ui.project_settings.atlas_width;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Height");

                if ui
                    .add(
                        egui::DragValue::new(&mut editor_ui.project_settings.atlas_height)
                            .range(1..=16384),
                    )
                    .changed()
                {
                    atlas.height = editor_ui.project_settings.atlas_height;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Padding");

                if ui
                    .add(
                        egui::DragValue::new(&mut editor_ui.project_settings.padding)
                            .range(0..=4096),
                    )
                    .changed()
                {
                    atlas.padding = editor_ui.project_settings.padding;
                }
            });

            ui.separator();

            ui.heading("Texture Naming");

            // Existing TextureNaming UI goes here.
        });

    editor_ui.show_project_settings_window = open;
}
