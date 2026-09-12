use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_system)
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        .run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn ui_example_system(mut contexts: EguiContexts) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Hello").show(ctx, |ui| {
            ui.label("world");
        });
    }
}
