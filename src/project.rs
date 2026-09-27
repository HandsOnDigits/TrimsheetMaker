use bevy::{asset::Assets, ecs::system::Commands, pbr::StandardMaterial, prelude::Mesh};

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    data::{Atlas, AtlasMaterial, MaterialTexture},
    mesh::{
        MeshPreview, MeshSceneState,
        import::{self, MeshImportSettings},
        viewport::{despawn_imported_mesh, spawn_mesh_scene},
    },
    project_settings::ProjectSettings,
    transform::Transform,
    ui::EditorUi,
};

pub const PROJECT_EXTENSION: &str = "tsm";
pub const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    pub format_version: u32,
    pub settings: ProjectSettings,
    pub mesh: Option<PathBuf>,
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
    pub fn from_state(
        project_path: &Path,
        atlas: &Atlas,
        settings: &ProjectSettings,
        mesh_path: Option<&Path>,
    ) -> Self {
        Self {
            format_version: PROJECT_FORMAT_VERSION,
            settings: settings.clone(),
            mesh: mesh_path.map(|mesh_path| make_project_relative_path(project_path, mesh_path)),
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
        mesh_path: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();

        let project = Self::from_state(path, atlas, settings, mesh_path);

        let config = ron::ser::PrettyConfig::new();

        let contents = ron::ser::to_string_pretty(&project, config)?;

        fs::write(path, contents)?;

        Ok(())
    }

    pub fn load(
        path: impl AsRef<Path>,
    ) -> Result<(Atlas, ProjectSettings, Option<PathBuf>), Box<dyn std::error::Error>> {
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

        Ok((atlas, project.settings, project.mesh))
    }
}

pub fn load_project_mesh(
    project_path: &Path,
    mesh_path: Option<&Path>,
    editor_ui: &mut EditorUi,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    mesh_scene_state: &mut MeshSceneState,
    mesh_preview: &mut MeshPreview,
) -> Result<(), Box<dyn std::error::Error>> {
    // Always remove the currently loaded mesh first.
    despawn_imported_mesh(commands, mesh_scene_state);

    let Some(mesh_path) = mesh_path else {
        editor_ui.mesh_import_path = None;
        return Ok(());
    };

    /*
     * Resolve relative mesh paths relative to the project file.
     *
     * Absolute paths are left untouched.
     */
    let resolved_path = if mesh_path.is_absolute() {
        mesh_path.to_path_buf()
    } else {
        project_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(mesh_path)
    };

    editor_ui.mesh_import_path = Some(resolved_path.clone());

    if !resolved_path.exists() {
        return Err(format!(
            "Saved mesh file does not exist: {}",
            resolved_path.display()
        )
        .into());
    }

    let settings = MeshImportSettings {
        assimp_library: editor_ui.editor_settings.assimp_library_path.clone(),
    };

    let scene = import::import(&resolved_path, editor_ui.mesh_import_backend, &settings)?;

    let (root, bounds_center, bounds_radius) =
        spawn_mesh_scene(commands, meshes, materials, &scene);

    mesh_scene_state.root = Some(root);
    mesh_scene_state.bounds_center = bounds_center;
    mesh_scene_state.bounds_radius = bounds_radius;

    /*
     * Frame the camera around the imported mesh.
     */
    mesh_preview.target = bounds_center;

    let fit_distance = (bounds_radius * 2.5).max(0.25);

    mesh_preview.distance = fit_distance;
    mesh_preview.target_distance = fit_distance;

    Ok(())
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
    mesh_path: Option<&Path>,
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

    ProjectFile::save(&path, atlas, settings, mesh_path)?;

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
    mesh_path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    ProjectFile::save(path, atlas, settings, mesh_path)
}

fn make_project_relative_path(project_path: &Path, asset_path: &Path) -> PathBuf {
    let Some(project_dir) = project_path.parent() else {
        return asset_path.to_path_buf();
    };

    if let Ok(relative) = asset_path.strip_prefix(project_dir) {
        relative.to_path_buf()
    } else {
        asset_path.to_path_buf()
    }
}
