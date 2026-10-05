use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod ui;
use ui::EditorUiPlugin;

mod data;
mod image;
mod mesh;

mod hotkeys;
use hotkeys::HotkeyPlugin;

mod editor_settings;
mod project;
mod project_settings;
mod rendering;
mod transform;
mod viewport;

fn main() {
    App::new()
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
        .add_plugins(viewport::MeshPreviewPlugin)
        .add_plugins(EditorUiPlugin)
        .run();
}
