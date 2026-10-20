use std::collections::{BTreeMap, HashSet};

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowCloseRequested},
};
use bevy_egui::{EguiGlobalSettings, EguiPrimaryContextPass, PrimaryEguiContext, egui};

use crate::{
    data::{self, Atlas},
    editor_settings::EditorSettings,
    history::{EditorHistory, EditorSnapshot},
    hotkeys::{self, EditorAction, EditorActionMessage, has_unsaved_project_content},
    mesh::{MeshLight, MeshSceneState},
    native_dialog::NativeDialogState,
    project_settings::ProjectSettings,
    transform::UvTransform,
    ui::{loading::LoadingState, windows::floating_context_menus::AtlasContextMenu},
};

pub mod mesh_import_task;
use mesh_import_task::{MeshImportTask, poll_mesh_import_task};

pub mod loading;
pub mod panels;
pub mod windows;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorUi>()
            .init_resource::<data::Atlas>()
            .init_resource::<AtlasContextMenu>()
            .init_resource::<NativeDialogState>()
            .init_resource::<LoadingState>()
            .init_resource::<MeshImportTask>()
            .add_systems(Startup, (setup, load_editor_settings))
            .add_systems(
                EguiPrimaryContextPass,
                (
                    apply_editor_actions.after(hotkeys::hotkey_system),
                    panels::editor_ui,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_window_close_requests,
                    update_window_title,
                    poll_mesh_import_task,
                ),
            );
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
    mut dialog: ResMut<NativeDialogState>,
    mut editor_ui: ResMut<EditorUi>,
    mut atlas: ResMut<data::Atlas>,
    mut images: ResMut<Assets<Image>>,
    mut mesh_preview: ResMut<crate::viewport::MeshPreview>,
    mut mesh_light: ResMut<MeshLight>,
    mut mesh_scene_state: ResMut<MeshSceneState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut history: ResMut<EditorHistory>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for message in messages.read() {
        hotkeys::dispatch_action(
            message.action,
            &mut dialog,
            &mut editor_ui,
            &mut atlas,
            &mut images,
            &mut mesh_preview,
            &mut mesh_light,
            &mut mesh_scene_state,
            &mut commands,
            &mut meshes,
            &mut history,
            &mut app_exit,
        );
    }
}

#[derive(Resource)]
pub struct EditorUi {
    pub show_unsaved_project_warning: bool,
    pub pending_project_action: Option<EditorAction>,
    pub pending_window_close: Option<Entity>,
    pub pending_app_exit: bool,
    pub approved_project_action: Option<EditorAction>,
    pub save_before_pending_action: bool,

    pub project_dirty: bool,
    pub project_path: Option<std::path::PathBuf>,

    pub atlas_drag_snapshot: Option<EditorSnapshot>,
    pub atlas_uv_drag_start: Option<Vec<(u64, usize, usize, UvTransform)>>,
    pub atlas_settings_history: Option<EditorSnapshot>,
    pub atlas_uv_rotation_drag: bool,
    pub atlas_uv_rotation_start_angle: Option<f32>,

    pub properties_snapshot: Option<EditorSnapshot>,

    pub selected_mesh: Option<u64>,
    pub selected_mesh_object: Option<(u64, usize)>,
    pub expanded_mesh_objects: HashSet<(u64, usize)>,
    pub selected_uv_assignment: Option<(u64, usize, usize)>,

    pub expanded_meshes: std::collections::BTreeSet<u64>,

    pub inspector_tab: InspectorTab,
    pub left_panel_tab: LeftPanelTab,

    pub rendering_revision: u64,

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

    /// Atlas region currently being inspected by the UI.
    ///
    /// This is not a material selection. It identifies which
    /// `AtlasMaterial` is shown in the inspector.
    pub inspected_atlas_region: Option<usize>,

    /// Atlas region currently being dragged by the mouse.
    ///
    /// This is transient interaction state and is independent
    /// from the inspector target.
    pub dragging_atlas_region: Option<usize>,

    pub drag_offset: egui::Vec2,

    pub snap_distance: f32,

    pub suffix_inputs: BTreeMap<String, String>,

    pub atlas_zoom: f32,
    pub atlas_pan: egui::Vec2,

    pub show_materials: bool,
    pub show_inspector: bool,
    pub show_status: bool,

    pub trim_to_edge: bool,
}

impl EditorUi {
    pub fn clear_mesh_selection(&mut self) {
        self.selected_mesh = None;
        self.selected_mesh_object = None;
        self.expanded_meshes.clear();
        self.expanded_mesh_objects.clear();
    }
}

impl Default for EditorUi {
    fn default() -> Self {
        Self {
            show_unsaved_project_warning: false,
            pending_project_action: None,
            pending_window_close: None,
            pending_app_exit: false,
            approved_project_action: None,
            save_before_pending_action: false,

            project_dirty: false,
            project_path: None,

            atlas_drag_snapshot: None,
            atlas_uv_drag_start: None,
            atlas_settings_history: None,
            atlas_uv_rotation_drag: false,
            atlas_uv_rotation_start_angle: None,

            properties_snapshot: None,
            selected_uv_assignment: None,

            selected_mesh: None,
            selected_mesh_object: None,
            expanded_mesh_objects: HashSet::new(),

            expanded_meshes: std::collections::BTreeSet::new(),

            inspector_tab: default(),
            left_panel_tab: default(),

            rendering_revision: 0,

            multiple_materials_root: None,
            import_materials: BTreeMap::new(),

            single_material_root: None,

            single_material_name: String::new(),
            single_material_textures: BTreeMap::new(),
            mesh_import_paths: Vec::new(),

            inspected_atlas_region: None,

            dragging_atlas_region: None,
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

            atlas_zoom: 1.0,
            atlas_pan: egui::Vec2::ZERO,

            show_materials: true,
            show_inspector: true,
            show_status: true,

            trim_to_edge: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InspectorTab {
    #[default]
    Properties,
    Rendering,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LeftPanelTab {
    #[default]
    Scene,
    Materials,
}

fn handle_window_close_requests(
    mut close_requests: MessageReader<WindowCloseRequested>,
    mut editor_ui: ResMut<EditorUi>,
    atlas: Res<Atlas>,
    mesh_scene_state: Res<MeshSceneState>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for _request in close_requests.read() {
        if has_unsaved_project_content(&editor_ui, &atlas, &mesh_scene_state) {
            editor_ui.pending_app_exit = true;
            editor_ui.show_unsaved_project_warning = true;
        } else {
            app_exit.write(AppExit::Success);
        }
    }
}

fn update_window_title(
    editor_ui: Res<EditorUi>,
    atlas: Res<Atlas>,
    mesh_scene_state: Res<MeshSceneState>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = window.single_mut() else {
        return;
    };

    let project_name = editor_ui
        .project_path
        .as_deref()
        .and_then(std::path::Path::file_stem)
        .map(|name| name.to_string_lossy().into_owned());

    let unsaved = has_unsaved_project_content(&editor_ui, &atlas, &mesh_scene_state);

    let title = match project_name {
        Some(name) => format!(
            "Trimsheet Maker — {name}{}",
            if unsaved { " *" } else { "" }
        ),
        None => format!("Trimsheet Maker{}", if unsaved { " *" } else { "" }),
    };

    if window.title != title {
        window.title = title;
    }
}
