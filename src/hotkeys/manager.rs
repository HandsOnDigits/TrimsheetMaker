use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::actions::{EditorAction, EditorActionMessage};

use crate::editor_settings::{EditorSettings, SavedHotkey};

/// A keyboard shortcut consisting of a key and optional modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hotkey {
    pub key: KeyCode,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub super_key: bool,
    pub delete: bool,
}

#[allow(unused)]
impl Hotkey {
    pub const fn new(
        key: KeyCode,
        ctrl: bool,
        shift: bool,
        alt: bool,
        super_key: bool,
        delete: bool,
    ) -> Self {
        Self {
            key,
            ctrl,
            shift,
            alt,
            super_key,
            delete,
        }
    }

    pub const fn plain(key: KeyCode) -> Self {
        Self::new(key, false, false, false, false, false)
    }

    pub const fn ctrl(key: KeyCode) -> Self {
        Self::new(key, true, false, false, false, false)
    }

    pub const fn ctrl_shift(key: KeyCode) -> Self {
        Self::new(key, true, true, false, false, false)
    }

    pub const fn shift(key: KeyCode) -> Self {
        Self::new(key, false, true, false, false, false)
    }

    pub const fn delete(key: KeyCode) -> Self {
        Self::new(key, false, false, false, false, true)
    }

    /// Returns a readable representation of the shortcut.
    pub fn display_name(&self) -> String {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("Ctrl");
        }

        if self.shift {
            parts.push("Shift");
        }

        if self.alt {
            parts.push("Alt");
        }

        if self.super_key {
            parts.push("Super");
        }

        if self.delete {
            parts.push("Delete");
        }

        parts.push(key_display_name(self.key));

        parts.join("+")
    }

    pub fn to_saved(&self) -> SavedHotkey {
        SavedHotkey {
            key: format!("{:?}", self.key),
            ctrl: self.ctrl,
            shift: self.shift,
            alt: self.alt,
            super_key: self.super_key,
            delete: self.delete,
        }
    }

    pub fn from_saved(saved: &SavedHotkey) -> Option<Self> {
        let key = parse_key_code(&saved.key)?;

        Some(Self::new(
            key,
            saved.ctrl,
            saved.shift,
            saved.alt,
            saved.super_key,
            saved.delete,
        ))
    }

    pub fn display_rich_text(&self, ui: &egui::Ui) -> egui::RichText {
        egui::RichText::new(self.display_name()).color(ui.visuals().weak_text_color())
    }
}

/// Stores the keyboard bindings for editor actions.
#[derive(Resource)]
pub struct HotkeyManager {
    bindings: HashMap<EditorAction, Hotkey>,
}

impl Default for HotkeyManager {
    fn default() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
        };

        manager.set_default_bindings();

        manager
    }
}

impl HotkeyManager {
    pub fn set_default_bindings(&mut self) {
        self.bindings.clear();

        // File
        self.bindings
            .insert(EditorAction::NewAtlas, Hotkey::ctrl(KeyCode::KeyN));

        self.bindings
            .insert(EditorAction::OpenProject, Hotkey::ctrl(KeyCode::KeyO));

        self.bindings
            .insert(EditorAction::SaveProject, Hotkey::ctrl(KeyCode::KeyS));

        self.bindings.insert(
            EditorAction::SaveProjectAs,
            Hotkey::ctrl_shift(KeyCode::KeyS),
        );

        self.bindings
            .insert(EditorAction::ExportAtlas, Hotkey::ctrl_shift(KeyCode::KeyE));

        // Edit
        self.bindings
            .insert(EditorAction::Undo, Hotkey::ctrl(KeyCode::KeyZ));

        self.bindings
            .insert(EditorAction::Redo, Hotkey::ctrl_shift(KeyCode::KeyZ));

        // Atlas
        self.bindings.insert(
            EditorAction::RemoveAtlasRegion,
            Hotkey::delete(KeyCode::KeyR),
        );

        // Mesh
        self.bindings.insert(
            EditorAction::RemoveSelectedMesh,
            Hotkey::delete(KeyCode::KeyM),
        );

        // View
        self.bindings
            .insert(EditorAction::ResetViewport, Hotkey::plain(KeyCode::KeyF));

        self.bindings.insert(
            EditorAction::ToggleMeshLightFollowCamera,
            Hotkey::shift(KeyCode::KeyL),
        );

        self.bindings
            .insert(EditorAction::ToggleMeshWindow, Hotkey::shift(KeyCode::KeyM));

        self.bindings.insert(
            EditorAction::ToggleMeshPartsVisibility,
            Hotkey::shift(KeyCode::KeyH),
        );

        self.bindings.insert(
            EditorAction::ToggleUvTransformMode,
            Hotkey::plain(KeyCode::KeyT),
        );

        self.bindings.insert(
            EditorAction::TrimAtlasExactDimensions,
            Hotkey::new(KeyCode::KeyT, true, false, true, false, false),
        );

        self.bindings.insert(
            EditorAction::TrimAtlasPowerOfTwo,
            Hotkey::new(KeyCode::KeyT, true, true, true, false, false),
        );

        self.bindings.insert(
            EditorAction::RepackAtlas,
            Hotkey::new(KeyCode::KeyP, true, false, true, false, false),
        );
    }

    pub fn binding(&self, action: EditorAction) -> Option<Hotkey> {
        self.bindings.get(&action).copied()
    }

    pub fn bindings(&self) -> &HashMap<EditorAction, Hotkey> {
        &self.bindings
    }

    /// Assigns a hotkey to an action.
    ///
    /// Returns an error if another action already uses
    /// the requested shortcut.
    pub fn set_binding(
        &mut self,
        action: EditorAction,
        hotkey: Hotkey,
    ) -> Result<(), HotkeyConflict> {
        if let Some(conflicting_action) =
            self.bindings
                .iter()
                .find_map(|(existing_action, existing_hotkey)| {
                    (*existing_action != action && *existing_hotkey == hotkey)
                        .then_some(*existing_action)
                })
        {
            return Err(HotkeyConflict {
                action,
                conflicting_action,
                hotkey,
            });
        }

        self.bindings.insert(action, hotkey);

        Ok(())
    }

    #[allow(unused)]
    pub fn remove_binding(&mut self, action: EditorAction) {
        self.bindings.remove(&action);
    }

    /// Finds the action assigned to a specific shortcut.
    #[allow(unused)]
    pub fn action_for_hotkey(&self, hotkey: Hotkey) -> Option<EditorAction> {
        self.bindings
            .iter()
            .find_map(|(action, existing_hotkey)| (*existing_hotkey == hotkey).then_some(*action))
    }

    pub fn display_name(&self, action: EditorAction) -> Option<String> {
        self.binding(action).map(|hotkey| hotkey.display_name())
    }

    pub fn capture_hotkey(
        &mut self,
        action: EditorAction,
        keyboard: &ButtonInput<KeyCode>,
    ) -> CaptureResult {
        if keyboard.just_pressed(KeyCode::Escape) {
            return CaptureResult::Cancelled;
        }

        let Some(key) = pressed_non_modifier_key(keyboard) else {
            return CaptureResult::None;
        };

        let hotkey = Hotkey::new(
            key,
            keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight),
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight),
            keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight),
            keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight),
            keyboard.pressed(KeyCode::Delete),
        );

        match self.set_binding(action, hotkey) {
            Ok(()) => CaptureResult::Assigned(hotkey),
            Err(conflict) => CaptureResult::Conflict(conflict),
        }
    }

    pub fn load_from_settings(&mut self, settings: &EditorSettings) {
        // Start with all current defaults.
        self.set_default_bindings();

        // Override defaults with the user's saved bindings.
        for (&action, saved) in &settings.hotkeys {
            if let Some(hotkey) = Hotkey::from_saved(saved) {
                self.bindings.insert(action, hotkey);
            }
        }
    }

    pub fn save_to_settings(&self, settings: &mut crate::editor_settings::EditorSettings) {
        settings.hotkeys = self
            .bindings
            .iter()
            .map(|(&action, &hotkey)| (action, hotkey.to_saved()))
            .collect();
    }

    // Renders an egui menu button with automatic right-aligned gray shortcut text.
    pub fn menu_button(&self, ui: &mut egui::Ui, action: EditorAction, label: &str) -> bool {
        let mut button = egui::Button::new(label);

        if let Some(shortcut) = self.display_name(action) {
            button = button.shortcut_text(shortcut);
        }

        ui.add(button).clicked()
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum CaptureResult {
    None,
    Cancelled,
    Assigned(Hotkey),
    Conflict(HotkeyConflict),
}

fn pressed_non_modifier_key(keyboard: &ButtonInput<KeyCode>) -> Option<KeyCode> {
    for &key in all_capture_keys() {
        if keyboard.just_pressed(key) {
            return Some(key);
        }
    }

    None
}

fn all_capture_keys() -> &'static [KeyCode] {
    &[
        KeyCode::Escape,
        KeyCode::Space,
        KeyCode::Enter,
        KeyCode::Tab,
        KeyCode::Backspace,
        KeyCode::Delete,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::KeyA,
        KeyCode::KeyB,
        KeyCode::KeyC,
        KeyCode::KeyD,
        KeyCode::KeyE,
        KeyCode::KeyF,
        KeyCode::KeyG,
        KeyCode::KeyH,
        KeyCode::KeyI,
        KeyCode::KeyJ,
        KeyCode::KeyK,
        KeyCode::KeyL,
        KeyCode::KeyM,
        KeyCode::KeyN,
        KeyCode::KeyO,
        KeyCode::KeyP,
        KeyCode::KeyQ,
        KeyCode::KeyR,
        KeyCode::KeyS,
        KeyCode::KeyT,
        KeyCode::KeyU,
        KeyCode::KeyV,
        KeyCode::KeyW,
        KeyCode::KeyX,
        KeyCode::KeyY,
        KeyCode::KeyZ,
        KeyCode::F1,
        KeyCode::F2,
        KeyCode::F3,
        KeyCode::F4,
        KeyCode::F5,
        KeyCode::F6,
        KeyCode::F7,
        KeyCode::F8,
        KeyCode::F9,
        KeyCode::F10,
        KeyCode::F11,
        KeyCode::F12,
    ]
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub struct HotkeyConflict {
    pub action: EditorAction,
    pub conflicting_action: EditorAction,
    pub hotkey: Hotkey,
}

/// Detects keyboard shortcuts and emits editor action messages.
///
/// Keyboard shortcuts are ignored while egui wants keyboard input,
/// such as when the user is typing into a text field.
pub fn hotkey_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    hotkeys: Res<HotkeyManager>,
    mut action_writer: MessageWriter<EditorActionMessage>,
    mut contexts: EguiContexts,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    if ctx.egui_wants_keyboard_input() {
        return;
    }

    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    let alt = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

    let super_key = keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight);

    let delete = keyboard.pressed(KeyCode::Delete);

    let modifiers = (ctrl, shift, alt, super_key, delete);

    for (&action, &hotkey) in hotkeys.bindings() {
        if !keyboard.just_pressed(hotkey.key) {
            continue;
        }

        let expected_modifiers = (
            hotkey.ctrl,
            hotkey.shift,
            hotkey.alt,
            hotkey.super_key,
            hotkey.delete,
        );

        if modifiers == expected_modifiers {
            action_writer.write(EditorActionMessage::new(action));
        }
    }
}

/// Converts a key code to a readable label.
fn key_display_name(key: KeyCode) -> &'static str {
    match key {
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Escape => "Escape",
        KeyCode::Tab => "Tab",
        KeyCode::Backspace => "Backspace",
        KeyCode::Delete => "Delete",

        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",

        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",

        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",

        _ => "Unknown",
    }
}

fn parse_key_code(value: &str) -> Option<KeyCode> {
    Some(match value {
        "Space" => KeyCode::Space,
        "Enter" => KeyCode::Enter,
        "Escape" => KeyCode::Escape,
        "Tab" => KeyCode::Tab,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,

        "ArrowUp" => KeyCode::ArrowUp,
        "ArrowDown" => KeyCode::ArrowDown,
        "ArrowLeft" => KeyCode::ArrowLeft,
        "ArrowRight" => KeyCode::ArrowRight,

        "KeyA" => KeyCode::KeyA,
        "KeyB" => KeyCode::KeyB,
        "KeyC" => KeyCode::KeyC,
        "KeyD" => KeyCode::KeyD,
        "KeyE" => KeyCode::KeyE,
        "KeyF" => KeyCode::KeyF,
        "KeyG" => KeyCode::KeyG,
        "KeyH" => KeyCode::KeyH,
        "KeyI" => KeyCode::KeyI,
        "KeyJ" => KeyCode::KeyJ,
        "KeyK" => KeyCode::KeyK,
        "KeyL" => KeyCode::KeyL,
        "KeyM" => KeyCode::KeyM,
        "KeyN" => KeyCode::KeyN,
        "KeyO" => KeyCode::KeyO,
        "KeyP" => KeyCode::KeyP,
        "KeyQ" => KeyCode::KeyQ,
        "KeyR" => KeyCode::KeyR,
        "KeyS" => KeyCode::KeyS,
        "KeyT" => KeyCode::KeyT,
        "KeyU" => KeyCode::KeyU,
        "KeyV" => KeyCode::KeyV,
        "KeyW" => KeyCode::KeyW,
        "KeyX" => KeyCode::KeyX,
        "KeyY" => KeyCode::KeyY,
        "KeyZ" => KeyCode::KeyZ,

        "F1" => KeyCode::F1,
        "F2" => KeyCode::F2,
        "F3" => KeyCode::F3,
        "F4" => KeyCode::F4,
        "F5" => KeyCode::F5,
        "F6" => KeyCode::F6,
        "F7" => KeyCode::F7,
        "F8" => KeyCode::F8,
        "F9" => KeyCode::F9,
        "F10" => KeyCode::F10,
        "F11" => KeyCode::F11,
        "F12" => KeyCode::F12,

        _ => return None,
    })
}
