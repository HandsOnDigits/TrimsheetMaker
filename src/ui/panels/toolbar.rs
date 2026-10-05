use bevy::prelude::*;
use bevy_egui::egui;

use super::{EditorUi, menu_action_button};

use crate::{
    data::Atlas,
    hotkeys::{EditorAction, HotkeyManager, execute_action},
    mesh::{
        MeshLight, MeshSceneState,
        unwrap::{MeshUvMode, unwrap_current_mesh},
    },
    viewport::MeshPreview,
};

pub fn toolbar(
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
) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        ui.menu_button("File", |ui| {
            if menu_action_button(ui, hotkeys, EditorAction::NewAtlas, "New Atlas") {
                execute_action(
                    EditorAction::NewAtlas,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }

            if menu_action_button(ui, hotkeys, EditorAction::OpenProject, "Open Project…") {
                execute_action(
                    EditorAction::OpenProject,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }

            if menu_action_button(ui, hotkeys, EditorAction::SaveProject, "Save Project") {
                execute_action(
                    EditorAction::SaveProject,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }

            if menu_action_button(ui, hotkeys, EditorAction::SaveProjectAs, "Save Project As…") {
                execute_action(
                    EditorAction::SaveProjectAs,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }

            ui.separator();

            if menu_action_button(
                ui,
                hotkeys,
                EditorAction::ImportSingleMaterial,
                "Import single Material…",
            ) {
                execute_action(
                    EditorAction::ImportSingleMaterial,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }

            if menu_action_button(
                ui,
                hotkeys,
                EditorAction::ImportMultipleMaterials,
                "Import Multiple Materials…",
            ) {
                execute_action(
                    EditorAction::ImportMultipleMaterials,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }

            if menu_action_button(ui, hotkeys, EditorAction::ImportMesh, "Import meshes…") {
                execute_action(
                    EditorAction::ImportMesh,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }

            ui.separator();

            if menu_action_button(ui, hotkeys, EditorAction::ExportAtlas, "Export Atlas…") {
                execute_action(
                    EditorAction::ExportAtlas,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }
        });

        ui.separator();

        ui.menu_button("Edit", |ui| {
            if menu_action_button(
                ui,
                hotkeys,
                EditorAction::RemoveAtlasRegion,
                "Remove Selected",
            ) {
                execute_action(
                    EditorAction::RemoveAtlasRegion,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }

            if menu_action_button(ui, hotkeys, EditorAction::ResetViewport, "Reset Viewports") {
                execute_action(
                    EditorAction::ResetViewport,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }
        });

        ui.separator();

        ui.label("UVs:");

        egui::ComboBox::from_id_salt("mesh_uv_mode")
            .selected_text(match mesh_preview.uv_mode {
                MeshUvMode::Imported => "Imported",
                MeshUvMode::Unwrapped => "Unwrapped",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut mesh_preview.uv_mode, MeshUvMode::Imported, "Imported");

                if mesh_preview.unwrapped_mesh_id.is_some() {
                    ui.selectable_value(
                        &mut mesh_preview.uv_mode,
                        MeshUvMode::Unwrapped,
                        "Unwrapped",
                    );
                }
            });

        if ui.button("Unwrap UVs").clicked() {
            if let Some(mesh_id) = mesh_preview.selected_mesh_id {
                if let Some(mesh) = mesh_scene_state.get(mesh_id) {
                    if let Err(error) = unwrap_current_mesh(mesh_preview, mesh_id, &mesh.scene) {
                        error!("Failed to unwrap mesh: {error:#}");
                    }
                }
            }
        }

        ui.separator();

        ui.menu_button("View", |ui| {
            ui.checkbox(&mut editor_ui.show_materials, "Materials");
            ui.checkbox(&mut editor_ui.show_inspector, "Inspector");
            ui.checkbox(&mut editor_ui.show_status, "Status Bar");

            ui.separator();

            if menu_action_button(ui, hotkeys, EditorAction::ResetViewport, "Reset Viewports") {
                execute_action(
                    EditorAction::ResetViewport,
                    editor_ui,
                    atlas,
                    images,
                    mesh_preview,
                    mesh_light,
                    mesh_scene_state,
                    commands,
                    meshes,
                );
                ui.close();
            }
        });

        ui.separator();

        egui::containers::menu::MenuButton::new("Project")
            .config(
                egui::containers::menu::MenuConfig::new()
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
            )
            .ui(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.set_width(220.0);

                    ui.label("Atlas");

                    ui.separator();

                    egui::Grid::new("project_atlas_settings")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Width");

                            ui.add_sized(
                                [100.0, 20.0],
                                egui::DragValue::new(&mut editor_ui.project_settings.atlas_width)
                                    .range(1..=16384)
                                    .speed(1.0),
                            );

                            ui.end_row();

                            ui.label("Height");

                            ui.add_sized(
                                [100.0, 20.0],
                                egui::DragValue::new(&mut editor_ui.project_settings.atlas_height)
                                    .range(1..=16384)
                                    .speed(1.0),
                            );

                            ui.end_row();

                            ui.label("Padding");

                            ui.add_sized(
                                [100.0, 20.0],
                                egui::DragValue::new(&mut editor_ui.project_settings.padding)
                                    .range(0..=4096)
                                    .speed(1.0),
                            );

                            ui.end_row();
                        });
                });

                editor_ui.project_settings.apply_to_atlas(atlas);
            });

        ui.separator();

        ui.menu_button("Settings", |ui| {
            if ui.button("Editor").clicked() {
                editor_ui.show_editor_settings_window = true;
                ui.close();
            }
        });

        ui.separator();

        ui.label(format!("Atlas: {} × {}", atlas.width, atlas.height));
    });
}
