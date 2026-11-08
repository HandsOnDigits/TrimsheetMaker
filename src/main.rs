use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowMode},
};
use bevy_egui::EguiPlugin;

mod ui;
use ui::EditorUiPlugin;

mod data;
mod editor_settings;

mod history;
use history::EditorHistory;

mod export;
mod image;
mod linking;
mod mesh;
mod native_dialog;
mod project;
mod project_settings;
mod rendering;
mod transform;
mod window_icon;

mod viewport;
use viewport::MeshPreviewPlugin;

mod hotkeys;
use hotkeys::HotkeyPlugin;

fn main() {
    App::new()
        .insert_resource(EditorHistory::default())
        .init_resource::<native_dialog::NativeDialogState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            close_when_requested: false,
            primary_window: Some(Window {
                title: "Trimsheet Maker".into(),
                resolution: (1400, 900).into(),
                resizable: true,
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(HotkeyPlugin)
        .add_plugins(MeshPreviewPlugin)
        .add_plugins(EditorUiPlugin)
        .add_systems(Startup, maximize_window)
        .add_systems(Last, window_icon::set_window_icon)
        .run();
}

fn maximize_window(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.single_mut() {
        window.set_maximized(true);
    }
}
