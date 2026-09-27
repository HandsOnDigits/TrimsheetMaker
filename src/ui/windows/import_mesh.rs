use std::path::PathBuf;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    mesh::{
        MeshPreview, MeshSceneState,
        import::{self, MeshBackend, MeshImportSettings},
        viewport::spawn_mesh_scene,
    },
    ui::EditorUi,
};

use super::FloatingWindowInteraction;

pub fn import_mesh_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    mesh_scene_state: &mut MeshSceneState,
    mesh_preview: &mut MeshPreview,
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
                    let paths = rfd::FileDialog::new()
                        .add_filter(
                            "Mesh Files",
                            &["gltf", "glb", "fbx", "obj", "dae", "3ds", "ply", "stl"],
                        )
                        .pick_files();

                    if let Some(paths) = paths {
                        editor_ui.mesh_import_paths = paths;
                        editor_ui.mesh_import_error = None;
                    }
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

                let can_import = !editor_ui.mesh_import_paths.is_empty();

                if ui
                    .add_enabled(can_import, egui::Button::new("Import"))
                    .clicked()
                {
                    let paths = editor_ui.mesh_import_paths.clone();

                    let settings = MeshImportSettings {
                        assimp_library: editor_ui.editor_settings.assimp_library_path.clone(),
                    };

                    let mut imported = 0usize;
                    let mut errors = Vec::new();

                    for path in paths {
                        match import::import(&path, editor_ui.mesh_import_backend, &settings) {
                            Ok(scene) => {
                                // Reserve the ID before spawning the entity so the
                                // ImportedMeshRoot receives the same ID as MeshEntry.
                                let mesh_id = mesh_scene_state.next_id;

                                let (root, parts, bounds_center, bounds_radius) =
                                    spawn_mesh_scene(commands, meshes, materials, &scene, mesh_id);

                                let name = path
                                    .file_stem()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or("Mesh")
                                    .to_owned();

                                mesh_scene_state.add(
                                    name,
                                    PathBuf::from(&path),
                                    root,
                                    parts,
                                    bounds_center,
                                    bounds_radius,
                                );

                                imported += 1;
                            }

                            Err(error) => {
                                errors.push(format!("{}: {error}", path.display()));
                            }
                        }
                    }

                    if imported > 0 {
                        if let Some((bounds_center, bounds_radius)) =
                            mesh_scene_state.visible_bounds()
                        {
                            mesh_preview.target = bounds_center;

                            let fit_distance = (bounds_radius * 2.5).max(0.25);

                            mesh_preview.distance = fit_distance;
                            mesh_preview.target_distance = fit_distance;
                        }
                    }

                    if errors.is_empty() {
                        editor_ui.mesh_import_error = None;
                        editor_ui.show_import_mesh_window = false;
                    } else {
                        editor_ui.mesh_import_error = Some(format!(
                            "Imported {imported} mesh{}.\n\n{}",
                            if imported == 1 { "" } else { "es" },
                            errors.join("\n")
                        ));

                        // Keep the window open so the user can see which
                        // files failed.
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
