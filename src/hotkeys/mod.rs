mod actions;
mod manager;

pub use actions::{
    EditorAction, EditorActionMessage, dispatch_action, execute_action,
    handle_native_dialog_result, has_unsaved_project_content,
};

#[allow(unused)]
pub use manager::{CaptureResult, Hotkey, HotkeyConflict, HotkeyManager, hotkey_system};

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HotkeyManager>()
            .add_message::<EditorActionMessage>()
            .add_systems(EguiPrimaryContextPass, hotkey_system);
    }
}
