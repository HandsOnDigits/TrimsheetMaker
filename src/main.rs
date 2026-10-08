use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod ui;
use ui::EditorUiPlugin;

mod data;
mod editor_settings;

mod history;
use history::EditorHistory;

mod image;
mod mesh;
mod project;
mod project_settings;
mod rendering;
mod transform;

mod viewport;
use viewport::MeshPreviewPlugin;

mod hotkeys;
use hotkeys::HotkeyPlugin;

fn main() {
    App::new()
        .insert_resource(EditorHistory::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Trimsheet Maker".into(),
                resolution: (1400, 900).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(HotkeyPlugin)
        .add_plugins(MeshPreviewPlugin)
        .add_plugins(EditorUiPlugin)
        .run();
}
