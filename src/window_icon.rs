use bevy::{ecs::system::NonSendMarker, prelude::*, winit::WINIT_WINDOWS};

use winit::window::Icon;

pub fn set_window_icon(_marker: NonSendMarker) {
    let image = match image::open("assets/icon.png") {
        Ok(image) => image.into_rgba8(),
        Err(error) => {
            error!("Failed to load application icon: {error}");
            return;
        }
    };

    let (width, height) = image.dimensions();

    let icon = match Icon::from_rgba(image.into_raw(), width, height) {
        Ok(icon) => icon,
        Err(error) => {
            error!("Failed to create application icon: {error}");
            return;
        }
    };

    WINIT_WINDOWS.with_borrow(|windows| {
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    });
}
