use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    mesh::{
        MeshPreview, MeshSceneState,
        import::{self, MeshBackend, MeshImportSettings},
        viewport::{despawn_imported_mesh, spawn_mesh_scene},
    },
    ui::EditorUi,
};

pub fn import_mesh_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    mesh_scene_state: &mut MeshSceneState,
    mesh_preview: &mut MeshPreview,
) {
    if !editor_ui.show_import_mesh_window {
        return;
    }

    let mut open = editor_ui.show_import_mesh_window;

    egui::Window::new("Import Mesh")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Mesh File");

            ui.horizontal(|ui| {
                let mut path_text = editor_ui
                    .mesh_import_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default();

                if ui
                    .add(egui::TextEdit::singleline(&mut path_text).desired_width(350.0))
                    .changed()
                {
                    editor_ui.mesh_import_error = None;

                    editor_ui.mesh_import_path = if path_text.trim().is_empty() {
                        None
                    } else {
                        Some(path_text.into())
                    };
                }

                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter(
                            "Mesh Files",
                            &["gltf", "glb", "fbx", "obj", "dae", "3ds", "ply", "stl"],
                        )
                        .pick_file()
                    {
                        editor_ui.mesh_import_path = Some(path);
                        editor_ui.mesh_import_error = None;
                    }
                }
            });

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

            ui.separator();

            if let Some(error) = &editor_ui.mesh_import_error {
                ui.separator();

                ui.colored_label(egui::Color32::RED, error);
            }

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    editor_ui.show_import_mesh_window = false;
                }

                let can_import = editor_ui.mesh_import_path.is_some();

                if ui
                    .add_enabled(can_import, egui::Button::new("Import"))
                    .clicked()
                {
                    let Some(path) = editor_ui.mesh_import_path.clone() else {
                        return;
                    };

                    let settings = MeshImportSettings {
                        assimp_library: editor_ui.editor_settings.assimp_library_path.clone(),
                    };

                    match import::import(&path, editor_ui.mesh_import_backend, &settings) {
                        Ok(scene) => {
                            despawn_imported_mesh(commands, mesh_scene_state);

                            let (root, bounds_center, bounds_radius) =
                                spawn_mesh_scene(commands, meshes, materials, &scene);

                            mesh_scene_state.root = Some(root);
                            mesh_scene_state.bounds_center = bounds_center;
                            mesh_scene_state.bounds_radius = bounds_radius;

                            mesh_scene_state.root = Some(root);

                            mesh_preview.target = bounds_center;

                            let fit_distance = (bounds_radius * 2.5).max(0.25);

                            mesh_preview.distance = fit_distance;
                            mesh_preview.target_distance = fit_distance;

                            editor_ui.mesh_import_error = None;
                            editor_ui.show_import_mesh_window = false;
                        }

                        Err(error) => {
                            editor_ui.mesh_import_error = Some(format!(
                                "Failed to import mesh '{}': {error}",
                                path.display()
                            ));
                        }
                    }
                }
            });
        });

    editor_ui.show_import_mesh_window = open;

    if !open {
        editor_ui.mesh_import_error = None;
    }
}
