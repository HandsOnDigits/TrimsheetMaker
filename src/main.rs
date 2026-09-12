use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod ui;
use ui::EditorUiPlugin;

mod assets;
mod data;
mod importer;

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
        .add_plugins(EditorUiPlugin)
        .run();
}
