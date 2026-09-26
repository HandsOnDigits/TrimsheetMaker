use bevy::prelude::Resource;

use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::hotkeys::EditorAction;

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct EditorSettings {
    pub mouse_scroll_speed: f32,

    #[serde(default)]
    pub hotkeys: HashMap<EditorAction, SavedHotkey>,

    pub assimp_library_path: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedHotkey {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            mouse_scroll_speed: 20.0,
            hotkeys: HashMap::new(),
            assimp_library_path: None,
        }
    }
}

impl EditorSettings {
    pub fn load() -> Self {
        let path = Self::file_path();

        let Ok(contents) = fs::read_to_string(&path) else {
            let mut settings = Self::default();

            let hotkeys = crate::hotkeys::HotkeyManager::default();
            hotkeys.save_to_settings(&mut settings);

            settings.save();

            return settings;
        };

        match ron::from_str(&contents) {
            Ok(settings) => settings,
            Err(error) => {
                eprintln!("Failed to load editor settings from {:?}: {}", path, error);

                let mut settings = Self::default();

                let hotkeys = crate::hotkeys::HotkeyManager::default();
                hotkeys.save_to_settings(&mut settings);

                settings.save();

                settings
            }
        }
    }

    pub fn save(&self) {
        let path = Self::file_path();

        let pretty = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true)
            .enumerate_arrays(true);

        let Ok(contents) = ron::ser::to_string_pretty(self, pretty) else {
            eprintln!("Failed to serialize editor settings.");
            return;
        };

        if let Err(error) = fs::write(&path, contents) {
            eprintln!("Failed to save editor settings to {:?}: {}", path, error);
        }
    }

    fn file_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("editor_settings.ron")
    }
}
