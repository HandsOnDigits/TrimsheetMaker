use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    mesh::import::{MeshBackend, MeshImportSettings},
    native_dialog::{NativeDialogRequest, NativeDialogState},
    ui::{EditorUi, loading::LoadingState, mesh_import_task::MeshImportTask},
};

use super::FloatingWindowInteraction;

pub fn import_mesh_window(
    ctx: &egui::Context,
    dialog: &mut NativeDialogState,
    editor_ui: &mut EditorUi,
    task: &mut MeshImportTask,
    loading: &mut LoadingState,
) -> bool {
    let mut open = editor_ui.show_import_mesh_window;

    let window_response = egui::Window::new("Import Meshes")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Mesh Files");

            ui.horizontal(|ui| {
                let path_text = if editor_ui.mesh_import_paths.is_empty() {
                    String::new()
                } else if editor_ui.mesh_import_paths.len() == 1 {
                    editor_ui.mesh_import_paths[0].display().to_string()
                } else {
                    format!("{} files selected", editor_ui.mesh_import_paths.len())
                };

                ui.add(
                    egui::TextEdit::singleline(&mut path_text.clone())
                        .desired_width(350.0)
                        .interactive(false),
                );

                if ui.button("Browse...").clicked() {
                    dialog.start(NativeDialogRequest::MeshFiles);
                }
            });

            if !editor_ui.mesh_import_paths.is_empty() {
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for path in &editor_ui.mesh_import_paths {
                            ui.label(path.display().to_string());
                        }
                    });
            }

            ui.separator();

            ui.label("Backend");

            let mut backend_changed = false;

            backend_changed |= ui
                .radio_value(
                    &mut editor_ui.mesh_import_backend,
                    MeshBackend::Auto,
                    "Auto",
                )
                .changed();

            backend_changed |= ui
                .radio_value(
                    &mut editor_ui.mesh_import_backend,
                    MeshBackend::Gltf,
                    "glTF",
                )
                .changed();

            backend_changed |= ui
                .radio_value(
                    &mut editor_ui.mesh_import_backend,
                    MeshBackend::Assimp,
                    "Assimp",
                )
                .changed();

            if backend_changed {
                editor_ui.mesh_import_error = None;
            }

            if let Some(error) = &editor_ui.mesh_import_error {
                ui.separator();
                ui.colored_label(egui::Color32::RED, error);
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    editor_ui.show_import_mesh_window = false;
                }

                let can_import = !editor_ui.mesh_import_paths.is_empty()
                    && !task.is_running()
                    && !loading.is_loading();

                if ui
                    .add_enabled(can_import, egui::Button::new("Import"))
                    .clicked()
                {
                    let paths = editor_ui.mesh_import_paths.clone();

                    let settings = MeshImportSettings {
                        assimp_library: editor_ui.editor_settings.assimp_library_path.clone(),
                    };

                    if !task.start(
                        paths,
                        editor_ui.mesh_import_backend.clone(),
                        settings,
                        loading,
                    ) {
                        editor_ui.mesh_import_error =
                            Some("A mesh import is already running.".to_string());
                    }
                }
            });
        });

    let owns_pointer = window_response
        .as_ref()
        .map(|response| response.owns_pointer(ctx))
        .unwrap_or(false);

    editor_ui.show_import_mesh_window = open;

    owns_pointer
}
