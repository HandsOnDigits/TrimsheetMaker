use bevy::prelude::*;
use bevy_egui::egui;

use rfd::FileDialog;

use crate::{
    mesh::{
        MeshSceneState,
        import::{self, MeshBackend, MeshImportSettings},
        viewport::{despawn_imported_mesh, spawn_mesh_scene},
    },
    platform,
    ui::EditorUi,
};

pub fn import_mesh_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    mesh_scene_state: &mut MeshSceneState,
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
                    editor_ui.mesh_import_path = if path_text.trim().is_empty() {
                        None
                    } else {
                        Some(path_text.into())
                    };
                }

                if ui.button("Browse...").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter(
                            "Mesh Files",
                            &["gltf", "glb", "fbx", "obj", "dae", "3ds", "ply", "stl"],
                        )
                        .pick_file()
                    {
                        editor_ui.mesh_import_path = Some(path);
                    }
                }
            });

            ui.separator();

            ui.label("Backend");

            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut editor_ui.mesh_import_backend,
                    MeshBackend::Auto,
                    "Auto",
                );

                ui.radio_value(
                    &mut editor_ui.mesh_import_backend,
                    MeshBackend::Gltf,
                    "glTF",
                );

                ui.radio_value(
                    &mut editor_ui.mesh_import_backend,
                    MeshBackend::Assimp,
                    "Assimp",
                );
            });

            ui.separator();

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

                    let assimp_library = platform::default_assimp_library_paths()
                        .into_iter()
                        .find(|path| path.is_file());

                    let settings = MeshImportSettings { assimp_library };

                    match import::import(&path, editor_ui.mesh_import_backend, &settings) {
                        Ok(scene) => {
                            despawn_imported_mesh(commands, mesh_scene_state);

                            let root = spawn_mesh_scene(commands, meshes, materials, &scene);

                            mesh_scene_state.root = Some(root);
                            editor_ui.show_import_mesh_window = false;
                        }

                        Err(error) => {
                            eprintln!("Failed to import mesh '{}': {error}", path.display());
                        }
                    }
                }
            });
        });

    editor_ui.show_import_mesh_window = open;
}
