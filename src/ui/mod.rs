use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPrimaryContextPass, PrimaryEguiContext, egui};

use crate::{
    data,
    editor_settings::EditorSettings,
    hotkeys::{self, EditorAction, EditorActionMessage},
    mesh::{MeshLight, MeshSceneState},
    project_settings::ProjectSettings,
};

pub mod panels;
mod windows;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorUi>()
            .init_resource::<data::Atlas>()
            .add_systems(Startup, (setup, load_editor_settings))
            .add_systems(
                EguiPrimaryContextPass,
                apply_editor_actions.after(hotkeys::hotkey_system),
            )
            .add_systems(EguiPrimaryContextPass, panels::editor_ui);
    }
}

fn load_editor_settings(editor_ui: Res<EditorUi>, mut hotkeys: ResMut<hotkeys::HotkeyManager>) {
    hotkeys.load_from_settings(&editor_ui.editor_settings);
}

fn setup(mut commands: Commands, mut egui_settings: ResMut<EguiGlobalSettings>) {
    egui_settings.auto_create_primary_context = false;

    commands.spawn((PrimaryEguiContext, Camera2d));
}

/// Applies actions emitted by the hotkey manager.
///
/// The toolbar can execute actions directly, while keyboard shortcuts
/// send messages that are handled here.
fn apply_editor_actions(
    mut messages: MessageReader<EditorActionMessage>,
    mut editor_ui: ResMut<EditorUi>,
    mut atlas: ResMut<data::Atlas>,
    mut images: ResMut<Assets<Image>>,
    mut mesh_preview: ResMut<crate::mesh::MeshPreview>,
    mut mesh_light: ResMut<MeshLight>,
    mut mesh_scene_state: ResMut<MeshSceneState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for message in messages.read() {
        hotkeys::execute_action(
            message.action,
            &mut editor_ui,
            &mut atlas,
            &mut images,
            &mut mesh_preview,
            &mut mesh_light,
            &mut mesh_scene_state,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }
}

#[derive(Resource)]
pub struct EditorUi {
    pub project_path: Option<std::path::PathBuf>,

    pub selected_material: Option<usize>,

    pub show_import_multiple_materials_window: bool,
    pub show_import_single_material_window: bool,
    pub show_editor_settings_window: bool,
    pub show_import_mesh_window: bool,

    pub mesh_import_path: Option<std::path::PathBuf>,
    pub mesh_import_backend: crate::mesh::import::MeshBackend,
    pub mesh_import_error: Option<String>,
    pub mesh_import_paths: Vec<std::path::PathBuf>,

    pub mesh_view_split: f32,

    pub hotkey_capture: Option<EditorAction>,

    pub editor_settings: EditorSettings,
    pub project_settings: ProjectSettings,

    pub multiple_materials_root: Option<std::path::PathBuf>,
    pub import_materials: BTreeMap<String, data::MaterialSource>,

    pub single_material_root: Option<std::path::PathBuf>,
    pub single_material_name: String,
    pub single_material_textures: BTreeMap<String, Option<std::path::PathBuf>>,

    pub dragging_material: Option<usize>,
    pub drag_offset: egui::Vec2,

    pub snap_distance: f32,

    pub suffix_inputs: BTreeMap<String, String>,

    pub atlas_zoom: f32,
    pub atlas_pan: egui::Vec2,

    pub show_materials: bool,
    pub show_inspector: bool,
    pub show_status: bool,
}

impl Default for EditorUi {
    fn default() -> Self {
        Self {
            project_path: None,

            multiple_materials_root: None,
            import_materials: BTreeMap::new(),

            single_material_root: None,

            single_material_name: String::new(),
            single_material_textures: BTreeMap::new(),
            mesh_import_paths: Vec::new(),

            dragging_material: None,
            drag_offset: egui::Vec2::ZERO,

            snap_distance: 8.0,

            show_import_multiple_materials_window: false,
            show_import_single_material_window: false,
            show_editor_settings_window: false,
            show_import_mesh_window: false,

            mesh_import_path: None,
            mesh_import_backend: crate::mesh::import::MeshBackend::Auto,
            mesh_import_error: None,

            mesh_view_split: 0.5,

            hotkey_capture: None,

            editor_settings: EditorSettings::load(),
            project_settings: ProjectSettings::default(),

            suffix_inputs: BTreeMap::new(),

            selected_material: None,

            atlas_zoom: 1.0,
            atlas_pan: egui::Vec2::ZERO,
            show_materials: true,
            show_inspector: true,
            show_status: true,
        }
    }
}
