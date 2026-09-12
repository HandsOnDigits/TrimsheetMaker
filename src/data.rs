use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureGroup {
    pub name: String,
    pub suffixes: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureNaming {
    pub groups: Vec<TextureGroup>,
}

impl TextureNaming {
    pub fn group_for_filename(&self, filename: &str) -> Option<String> {
        self.groups
            .iter()
            .flat_map(|group| group.suffixes.iter().map(move |suffix| (group, suffix)))
            .filter_map(|(group, suffix)| {
                filename
                    .to_ascii_lowercase()
                    .ends_with(&suffix.to_ascii_lowercase())
                    .then_some((suffix.len(), &group.name))
            })
            .max_by_key(|(length, _)| *length)
            .map(|(_, name)| name.clone())
    }
}

impl Default for TextureNaming {
    fn default() -> Self {
        Self {
            groups: vec![
                TextureGroup {
                    name: "Base Color".into(),
                    suffixes: vec!["_base_color".into()],
                },
                TextureGroup {
                    name: "Normal".into(),
                    suffixes: vec!["_normal".into()],
                },
                TextureGroup {
                    name: "Height".into(),
                    suffixes: vec!["_height".into()],
                },
                TextureGroup {
                    name: "ORM".into(),
                    suffixes: vec!["_orm".into()],
                },
            ],
        }
    }
}
