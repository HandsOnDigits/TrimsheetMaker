use bevy::prelude::*;
use bevy_egui::egui;

use super::EditorUi;

use crate::{
    data::Atlas,
    history::{EditorHistory, EditorSnapshot},
    hotkeys::{EditorAction, HotkeyManager, execute_action},
    mesh::{MeshLight, MeshSceneState},
    project_settings::HighlightMaterial,
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
    history: &mut EditorHistory,
) {
    egui::MenuBar::new()
        .config(
            egui::containers::menu::MenuConfig::new()
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
        )
        .ui(ui, |ui| {
            ui.add_space(8.0);

            // File
            ui.menu_button("File", |ui| {
                if hotkeys.menu_button(ui, EditorAction::NewAtlas, "New Atlas") {
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
                        history,
                    );
                    ui.close();
                }

                if hotkeys.menu_button(ui, EditorAction::OpenProject, "Open Project…") {
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
                        history,
                    );
                    ui.close();
                }

                if hotkeys.menu_button(ui, EditorAction::SaveProject, "Save Project") {
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
                        history,
                    );
                    ui.close();
                }

                if hotkeys.menu_button(ui, EditorAction::SaveProjectAs, "Save Project As…") {
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
                        history,
                    );
                    ui.close();
                }

                ui.separator();

                if hotkeys.menu_button(
                    ui,
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
                        history,
                    );
                    ui.close();
                }

                if hotkeys.menu_button(
                    ui,
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
                        history,
                    );
                    ui.close();
                }

                if hotkeys.menu_button(ui, EditorAction::ImportMesh, "Import meshes…") {
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
                        history,
                    );
                    ui.close();
                }

                ui.separator();

                if hotkeys.menu_button(ui, EditorAction::ExportAtlas, "Export Atlas…") {
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
                        history,
                    );
                    ui.close();
                }

                if hotkeys.menu_button(ui, EditorAction::ExportModel, "Export Model…") {
                    execute_action(
                        EditorAction::ExportModel,
                        editor_ui,
                        atlas,
                        images,
                        mesh_preview,
                        mesh_light,
                        mesh_scene_state,
                        commands,
                        meshes,
                        history,
                    );
                    ui.close();
                }
            });

            ui.separator();

            // Edit
            ui.menu_button("Edit", |ui| {
                if hotkeys.menu_button(ui, EditorAction::RemoveAtlasRegion, "Remove Selected") {
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
                        history,
                    );
                    ui.close();
                }

                ui.menu_button("Trim Atlas", |ui| {
                    ui.checkbox(&mut editor_ui.trim_to_edge, "Trim to Edge")
                        .on_hover_text(
                            "Remove unused outer margins while preserving gaps between regions.",
                        );

                    ui.separator();

                    // 1. Exact Dimensions
                    let mut exact_btn = egui::Button::new("Exact Dimensions");
                    if let Some(shortcut) =
                        hotkeys.display_name(EditorAction::TrimAtlasExactDimensions)
                    {
                        exact_btn = exact_btn.shortcut_text(shortcut);
                    }

                    if ui.add(exact_btn).clicked() {
                        execute_action(
                            EditorAction::TrimAtlasExactDimensions,
                            editor_ui,
                            atlas,
                            images,
                            mesh_preview,
                            mesh_light,
                            mesh_scene_state,
                            commands,
                            meshes,
                            history,
                        );
                        ui.close();
                    }

                    // 2. Power of Two
                    let mut power_of_two_btn = egui::Button::new("Power of Two");
                    if let Some(shortcut) = hotkeys.display_name(EditorAction::TrimAtlasPowerOfTwo)
                    {
                        power_of_two_btn = power_of_two_btn.shortcut_text(shortcut);
                    }

                    if ui.add(power_of_two_btn).clicked() {
                        execute_action(
                            EditorAction::TrimAtlasPowerOfTwo,
                            editor_ui,
                            atlas,
                            images,
                            mesh_preview,
                            mesh_light,
                            mesh_scene_state,
                            commands,
                            meshes,
                            history,
                        );
                        ui.close();
                    }
                });

                if hotkeys.menu_button(ui, EditorAction::RepackAtlas, "Repack Atlas") {
                    execute_action(
                        EditorAction::RepackAtlas,
                        editor_ui,
                        atlas,
                        images,
                        mesh_preview,
                        mesh_light,
                        mesh_scene_state,
                        commands,
                        meshes,
                        history,
                    );

                    ui.close();
                }
            });

            ui.separator();

            // View
            ui.menu_button("View", |ui| {
                ui.checkbox(&mut editor_ui.show_materials, "Materials");
                ui.checkbox(&mut editor_ui.show_inspector, "Inspector");
                ui.checkbox(&mut editor_ui.show_status, "Status Bar");

                ui.separator();

                if hotkeys.menu_button(ui, EditorAction::ResetViewport, "Reset Viewports") {
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
                        history,
                    );
                    ui.close();
                }
            });

            ui.separator();

            // Settings
            ui.menu_button("Settings", |ui| {
                ui.menu_button("Project", |ui| {
                    ui.set_min_width(280.0);

                    ui.label("Atlas");
                    ui.separator();

                    let settings_before_controls = editor_ui.project_settings.clone();

                    let (is_interacting, settings_changed) =
                        egui::Grid::new("project_atlas_settings")
                            .num_columns(2)
                            .spacing([12.0, 8.0])
                            .show(ui, |ui| {
                                ui.label("Width");
                                let width_response = ui.add_sized(
                                    [110.0, 20.0],
                                    egui::DragValue::new(
                                        &mut editor_ui.project_settings.atlas_width,
                                    )
                                    .range(1..=16384)
                                    .speed(1.0),
                                );
                                ui.end_row();

                                ui.label("Height");
                                let height_response = ui.add_sized(
                                    [110.0, 20.0],
                                    egui::DragValue::new(
                                        &mut editor_ui.project_settings.atlas_height,
                                    )
                                    .range(1..=16384)
                                    .speed(1.0),
                                );
                                ui.end_row();

                                ui.label("Padding");
                                let padding_response = ui.add_sized(
                                    [110.0, 20.0],
                                    egui::DragValue::new(&mut editor_ui.project_settings.padding)
                                        .range(0..=4096)
                                        .speed(1.0),
                                );
                                ui.end_row();

                                let responses =
                                    [&width_response, &height_response, &padding_response];

                                let is_interacting = responses
                                    .iter()
                                    .any(|response| response.dragged() || response.has_focus());

                                let settings_changed = editor_ui.project_settings.atlas_width
                                    != settings_before_controls.atlas_width
                                    || editor_ui.project_settings.atlas_height
                                        != settings_before_controls.atlas_height
                                    || editor_ui.project_settings.padding
                                        != settings_before_controls.padding;

                                (is_interacting, settings_changed)
                            })
                            .inner;

                    // Capture the state from BEFORE these controls changed the settings.
                    if settings_changed && editor_ui.atlas_settings_history.is_none() {
                        editor_ui.atlas_settings_history = Some(EditorSnapshot::capture(
                            atlas,
                            &settings_before_controls,
                            mesh_scene_state,
                        ));
                    }

                    // Apply the current settings after capturing the previous state.
                    editor_ui.project_settings.apply_to_atlas(atlas);

                    // Commit one history entry when the interaction ends.
                    if !is_interacting
                        && let Some(before) = editor_ui.atlas_settings_history.take()
                        && (atlas.width != before.atlas.width
                            || atlas.height != before.atlas.height
                            || atlas.padding != before.atlas.padding)
                    {
                        history.push(before);
                    }

                    ui.add_space(8.0);
                    ui.label("Highlights");
                    ui.separator();

                    egui::Grid::new("project_highlight_settings")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            highlight_controls(
                                ui,
                                "Mesh selection",
                                &mut editor_ui.project_settings.highlights.mesh,
                            );

                            highlight_controls(
                                ui,
                                "Atlas region",
                                &mut editor_ui.project_settings.highlights.atlas_region,
                            );

                            highlight_controls(
                                ui,
                                "UV selection",
                                &mut editor_ui.project_settings.highlights.uv,
                            );
                        });

                    ui.add_space(8.0);

                    // Keep atlas dimensions and padding in sync with the atlas.
                    editor_ui.project_settings.apply_to_atlas(atlas);
                });

                if ui.button("Editor…").clicked() {
                    editor_ui.show_editor_settings_window = true;
                    ui.close();
                }
            });

            ui.separator();

            ui.label(format!("Atlas: {} × {}", atlas.width, atlas.height));
        });
}

fn highlight_controls(ui: &mut egui::Ui, label: &str, highlight: &mut HighlightMaterial) {
    ui.label(label);
    ui.checkbox(&mut highlight.enabled, "Enabled");
    ui.end_row();

    ui.label("Color");

    let mut color = egui::Color32::from_rgba_unmultiplied(
        (highlight.color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (highlight.color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (highlight.color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (highlight.color[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    );

    egui::color_picker::color_picker_color32(ui, &mut color, egui::color_picker::Alpha::OnlyBlend);

    highlight.color = color.to_array().map(|channel| channel as f32 / 255.0);

    ui.end_row();

    ui.label("Strength");
    ui.add(egui::Slider::new(&mut highlight.strength, 0.0..=1.0).show_value(true));
    ui.end_row();
}
