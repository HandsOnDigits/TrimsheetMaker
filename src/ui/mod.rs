use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::{EguiPrimaryContextPass, egui};

use crate::{
    data,
    hotkeys::{self, EditorAction, EditorActionMessage},
};

mod panels;
mod windows;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorUi>()
            .init_resource::<data::Atlas>()
            .add_systems(Startup, setup)
            .add_systems(
                EguiPrimaryContextPass,
                apply_editor_actions.after(hotkeys::hotkey_system),
            )
            .add_systems(EguiPrimaryContextPass, panels::editor_ui);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Applies actions emitted by the hotkey manager.
///
/// The toolbar can execute actions directly, while keyboard shortcuts
/// send messages that are handled here.
fn apply_editor_actions(
    mut messages: MessageReader<EditorActionMessage>,
    mut editor_ui: ResMut<EditorUi>,
    mut atlas: ResMut<data::Atlas>,
) {
    for message in messages.read() {
        hotkeys::execute_action(message.action, &mut editor_ui, &mut atlas);
    }
}

#[derive(Resource)]
pub struct EditorUi {
    pub selected_material: Option<usize>,

    pub show_import_multiple_materials_window: bool,
    pub show_import_single_material_window: bool,
    pub show_editor_settings_window: bool,

    pub hotkey_capture: Option<EditorAction>,

    pub mouse_scroll_speed: f32,

    pub multiple_materials_root: Option<std::path::PathBuf>,
    pub import_materials: BTreeMap<String, data::MaterialSource>,

    pub single_material_root: Option<std::path::PathBuf>,
    pub single_material_name: String,
    pub single_material_textures: BTreeMap<String, Option<std::path::PathBuf>>,

    pub dragging_material: Option<usize>,
    pub drag_offset: egui::Vec2,

    pub snap_distance: f32,

    pub texture_naming: data::TextureNaming,
    pub suffix_inputs: BTreeMap<String, String>,

    // Texture group currently displayed in the atlas.
    pub preview_texture_group: Option<String>,

    pub atlas_zoom: f32,
    pub atlas_pan: egui::Vec2,

    pub show_materials: bool,
    pub show_inspector: bool,
    pub show_status: bool,
}

impl Default for EditorUi {
    fn default() -> Self {
        Self {
            multiple_materials_root: None,
            import_materials: BTreeMap::new(),

            single_material_root: None,

            single_material_name: String::new(),
            single_material_textures: BTreeMap::new(),

            dragging_material: None,
            drag_offset: egui::Vec2::ZERO,

            snap_distance: 8.0,

            show_import_multiple_materials_window: false,
            show_import_single_material_window: false,
            show_editor_settings_window: false,

            hotkey_capture: None,

            mouse_scroll_speed: 20.0,

            texture_naming: default(),
            suffix_inputs: BTreeMap::new(),

            selected_material: None,

            preview_texture_group: None,

            atlas_zoom: 1.0,
            atlas_pan: egui::Vec2::ZERO,
            show_materials: true,
            show_inspector: true,
            show_status: true,
        }
    }
}
