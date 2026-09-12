use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiPrimaryContextPass, egui};

use crate::data;

mod panels;
mod windows;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorUi>()
            .init_resource::<data::Atlas>()
            .add_systems(Startup, setup)
            .add_systems(EguiPrimaryContextPass, panels::editor_ui);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource)]
pub struct EditorUi {
    pub selected_material: Option<usize>,

    pub show_import_window: bool,

    pub import_root: Option<std::path::PathBuf>,
    pub import_materials: HashMap<String, data::MaterialSource>,

    pub texture_naming: data::TextureNaming,

    pub atlas_zoom: f32,
    pub atlas_pan: egui::Vec2,

    pub show_materials: bool,
    pub show_inspector: bool,
    pub show_status: bool,
}

impl Default for EditorUi {
    fn default() -> Self {
        Self {
            show_import_window: false,
            import_root: None,
            import_materials: HashMap::new(),
            texture_naming: default(),
            selected_material: Some(0),
            atlas_zoom: 1.0,
            atlas_pan: egui::Vec2::ZERO,
            show_materials: true,
            show_inspector: true,
            show_status: true,
        }
    }
}
