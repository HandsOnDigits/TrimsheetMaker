use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_systems(Update, ui_example_system)
        .run();
}

fn ui_example_system(mut contexts: EguiContexts) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Hello").show(ctx, |ui| {
            ui.label("world");
        });
    }
}
