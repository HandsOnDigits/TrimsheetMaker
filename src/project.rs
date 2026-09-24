use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    data::{Atlas, AtlasMaterial, MaterialTexture},
    project_settings::ProjectSettings,
    transform::Transform,
};

pub const PROJECT_EXTENSION: &str = "tsm";
pub const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    pub format_version: u32,
    pub settings: ProjectSettings,
    pub materials: Vec<ProjectMaterial>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMaterial {
    pub index: usize,
    pub name: String,
    pub textures: Vec<ProjectTexture>,
    pub transform: Transform,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectTexture {
    pub id: String,
    pub path: PathBuf,
}

impl ProjectFile {
    pub fn from_state(atlas: &Atlas, settings: &ProjectSettings) -> Self {
        Self {
            format_version: PROJECT_FORMAT_VERSION,
            settings: settings.clone(),
            materials: atlas
                .materials
                .iter()
                .map(|material| ProjectMaterial {
                    index: material.index,
                    name: material.name.clone(),
                    textures: material
                        .textures
                        .iter()
                        .map(ProjectTexture::from_texture)
                        .collect(),
                    transform: material.transform.clone(),
                })
                .collect(),
        }
    }

    pub fn save(
        path: impl AsRef<Path>,
        atlas: &Atlas,
        settings: &ProjectSettings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let project = Self::from_state(atlas, settings);

        let config = ron::ser::PrettyConfig::new();

        let contents = ron::ser::to_string_pretty(&project, config)?;

        fs::write(path, contents)?;

        Ok(())
    }

    pub fn load(
        path: impl AsRef<Path>,
    ) -> Result<(Atlas, ProjectSettings), Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;

        let project: Self = ron::from_str(&contents)?;

        if project.format_version != PROJECT_FORMAT_VERSION {
            return Err(format!(
                "Unsupported project format version: {}",
                project.format_version
            )
            .into());
        }

        let mut atlas = Atlas::default();

        atlas.width = project.settings.atlas_width;
        atlas.height = project.settings.atlas_height;
        atlas.padding = project.settings.padding;

        atlas.materials = project
            .materials
            .into_iter()
            .map(|material| AtlasMaterial {
                index: material.index,
                name: material.name,
                textures: material
                    .textures
                    .into_iter()
                    .map(|texture| MaterialTexture {
                        id: texture.id,
                        path: texture.path,
                        image: None,
                        egui_texture: None,
                    })
                    .collect(),
                transform: material.transform,
            })
            .collect();

        Ok((atlas, project.settings))
    }
}

impl ProjectTexture {
    fn from_texture(texture: &MaterialTexture) -> Self {
        Self {
            id: texture.id.clone(),
            path: texture.path.clone(),
        }
    }
}

pub fn save_project_as_dialog(
    atlas: &Atlas,
    settings: &ProjectSettings,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let Some(mut path) = rfd::FileDialog::new()
        .add_filter("Trimsheet Project", &[PROJECT_EXTENSION])
        .set_file_name("Untitled.tsm")
        .save_file()
    else {
        return Ok(None);
    };

    if path.extension().is_none() {
        path.set_extension(PROJECT_EXTENSION);
    }

    ProjectFile::save(&path, atlas, settings)?;

    Ok(Some(path))
}

pub fn open_project_dialog() -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Trimsheet Project", &[PROJECT_EXTENSION])
        .pick_file()
    else {
        return Ok(None);
    };

    Ok(Some(path))
}

pub fn save_project(
    path: &Path,
    atlas: &Atlas,
    settings: &ProjectSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    ProjectFile::save(path, atlas, settings)
}
