use bevy::prelude::*;

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    data::{Atlas, AtlasMaterial, MaterialMeshAssignment, MaterialTexture},
    mesh::{
        MeshSceneState,
        import::{self, MeshImportSettings},
        spawn_mesh_scene,
    },
    project_settings::ProjectSettings,
    transform::Transform,
    ui::EditorUi,
    viewport::MeshPreview,
};

pub const PROJECT_EXTENSION: &str = "tsm";
pub const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeshPart {
    pub object_index: usize,
    pub primitive_index: usize,

    #[serde(default = "default_mesh_visibility")]
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMesh {
    pub id: u64,
    pub name: String,
    pub path: PathBuf,

    #[serde(default = "default_mesh_visibility")]
    pub visible: bool,

    #[serde(default)]
    pub parts: Vec<ProjectMeshPart>,
}

fn default_mesh_visibility() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub format_version: u32,
    pub settings: ProjectSettings,
    pub meshes: Vec<ProjectMesh>,
    pub materials: Vec<ProjectMaterial>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectMaterial {
    pub index: usize,
    pub name: String,

    #[serde(default)]
    pub material_ref: Option<import::MeshMaterialRef>,

    pub textures: Vec<ProjectTexture>,
    pub transform: Transform,

    #[serde(default)]
    pub mesh_assignments: Vec<ProjectMaterialMeshAssignment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectTexture {
    pub id: String,

    #[serde(default)]
    pub group_id: Option<String>,

    pub path: PathBuf,
}

impl ProjectFile {
    pub fn from_state(
        project_path: &Path,
        atlas: &Atlas,
        settings: &ProjectSettings,
        mesh_scene_state: &MeshSceneState,
    ) -> Self {
        Self {
            format_version: PROJECT_FORMAT_VERSION,
            settings: settings.clone(),

            meshes: mesh_scene_state
                .meshes
                .iter()
                .map(|mesh| ProjectMesh {
                    id: mesh.id,
                    name: mesh.name.clone(),
                    path: make_project_relative_path(project_path, &mesh.path),
                    visible: mesh.visible,

                    parts: mesh
                        .parts
                        .iter()
                        .map(|part| ProjectMeshPart {
                            object_index: part.object_index,
                            primitive_index: part.primitive_index,
                            visible: part.visible,
                        })
                        .collect(),
                })
                .collect(),

            materials: atlas
                .materials
                .iter()
                .map(|material| ProjectMaterial {
                    index: material.index,
                    name: material.name.clone(),
                    material_ref: material.material_ref.clone(),
                    textures: material
                        .textures
                        .iter()
                        .map(ProjectTexture::from_texture)
                        .collect(),
                    transform: material.transform.clone(),

                    mesh_assignments: material
                        .mesh_assignments
                        .iter()
                        .map(|assignment| ProjectMaterialMeshAssignment {
                            mesh_id: assignment.mesh_id,
                            object_index: assignment.object_index,
                            primitive_index: assignment.primitive_index,
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    pub fn save(
        path: impl AsRef<Path>,
        atlas: &Atlas,
        settings: &ProjectSettings,
        mesh_scene_state: &MeshSceneState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();

        let project = Self::from_state(path, atlas, settings, mesh_scene_state);

        let config = ron::ser::PrettyConfig::new();

        let contents = ron::ser::to_string_pretty(&project, config)?;

        fs::write(path, contents)?;

        Ok(())
    }

    pub fn load(
        path: impl AsRef<Path>,
        images: &mut Assets<Image>,
    ) -> Result<(Atlas, ProjectSettings, Vec<ProjectMesh>), Box<dyn std::error::Error>> {
        let path = path.as_ref();

        let contents = fs::read_to_string(path)?;
        let project: Self = ron::from_str(&contents)?;

        if project.format_version != PROJECT_FORMAT_VERSION {
            return Err(format!(
                "Unsupported project format version: {}",
                project.format_version
            )
            .into());
        }

        let project_dir = path.parent().unwrap_or_else(|| Path::new("."));

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
                material_ref: material.material_ref,

                textures: material
                    .textures
                    .into_iter()
                    .map(|texture| {
                        let resolved_path = if texture.path.is_absolute() {
                            texture.path
                        } else {
                            project_dir.join(texture.path)
                        };

                        MaterialTexture {
                            id: texture.id,
                            group_id: texture.group_id,
                            path: resolved_path,
                            image: None,
                            egui_texture: None,
                        }
                    })
                    .collect(),

                transform: material.transform,

                mesh_assignments: material
                    .mesh_assignments
                    .into_iter()
                    .map(|assignment| MaterialMeshAssignment {
                        mesh_id: assignment.mesh_id,
                        object_index: assignment.object_index,
                        primitive_index: assignment.primitive_index,
                    })
                    .collect(),
            })
            .collect();

        atlas.load_textures(images);

        // Generated atlas images must be rebuilt after loading a project.
        atlas.images = Default::default();

        Ok((atlas, project.settings, project.meshes))
    }
}

pub fn load_project_meshes(
    project_path: &Path,
    project_meshes: &[ProjectMesh],
    editor_ui: &mut EditorUi,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mesh_scene_state: &mut MeshSceneState,
    mesh_preview: &mut MeshPreview,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::viewport::despawn_all_meshes(commands, mesh_scene_state);

    editor_ui.mesh_import_path = None;

    let settings = MeshImportSettings {
        assimp_library: editor_ui.editor_settings.assimp_library_path.clone(),
    };

    let mut errors = Vec::new();

    for project_mesh in project_meshes {
        let resolved_path = if project_mesh.path.is_absolute() {
            project_mesh.path.clone()
        } else {
            project_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(&project_mesh.path)
        };

        if !resolved_path.exists() {
            errors.push(format!(
                "{}: saved mesh file does not exist: {}",
                project_mesh.name,
                resolved_path.display()
            ));
            continue;
        }

        match import::import(&resolved_path, editor_ui.mesh_import_backend, &settings) {
            Ok(mut scene) => {
                let mesh_id = project_mesh.id;

                apply_saved_mesh_parts(&mut scene, &project_mesh.parts);

                let (root, parts, bounds_center, bounds_radius) = spawn_mesh_scene(
                    commands,
                    meshes,
                    &scene,
                    mesh_id,
                    mesh_preview.material.clone(),
                );

                mesh_scene_state.add_with_id(
                    project_mesh.id,
                    project_mesh.name.clone(),
                    resolved_path.clone(),
                    scene,
                    root,
                    parts,
                    bounds_center,
                    bounds_radius,
                );

                if let Some(mesh) = mesh_scene_state.get_mut(mesh_id) {
                    mesh.visible = project_mesh.visible;

                    for saved_part in &project_mesh.parts {
                        if let Some(part) = mesh.parts.iter_mut().find(|part| {
                            part.object_index == saved_part.object_index
                                && part.primitive_index == saved_part.primitive_index
                        }) {
                            part.visible = saved_part.visible;
                        }
                    }
                }
            }

            Err(error) => {
                errors.push(format!("{}: failed to import: {error}", project_mesh.name));
            }
        }
    }

    if let Some((bounds_center, bounds_radius)) = mesh_scene_state.visible_bounds() {
        mesh_preview.target = bounds_center;

        let fit_distance = (bounds_radius * 2.5).max(0.25);

        mesh_preview.distance = fit_distance;
        mesh_preview.target_distance = fit_distance;
    } else {
        mesh_preview.target = Vec3::ZERO;
        mesh_preview.distance = 4.0;
        mesh_preview.target_distance = 4.0;
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n").into())
    }
}

pub fn restore_history_meshes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mesh_scene_state: &mut MeshSceneState,
    mesh_preview: &mut MeshPreview,
) {
    let mesh_data = mesh_scene_state
        .meshes
        .iter()
        .map(|mesh| {
            (
                mesh.id,
                mesh.name.clone(),
                mesh.path.clone(),
                mesh.visible,
                mesh.scene.clone(),
            )
        })
        .collect::<Vec<_>>();

    mesh_scene_state.meshes.clear();
    mesh_scene_state.next_id = 0;

    for (id, name, path, visible, scene) in mesh_data {
        let (root, parts, bounds_center, bounds_radius) =
            spawn_mesh_scene(commands, meshes, &scene, id, mesh_preview.material.clone());

        mesh_scene_state.add_with_id(
            id,
            name,
            path,
            scene,
            root,
            parts,
            bounds_center,
            bounds_radius,
        );

        if let Some(mesh) = mesh_scene_state.get_mut(id) {
            mesh.visible = visible;
        }
    }

    if let Some((center, radius)) = mesh_scene_state.visible_bounds() {
        mesh_preview.target = center;

        let distance = (radius * 2.5).max(0.25);

        mesh_preview.distance = distance;
        mesh_preview.target_distance = distance;
    } else {
        mesh_preview.target = Vec3::ZERO;
        mesh_preview.distance = 4.0;
        mesh_preview.target_distance = 4.0;
    }

    mesh_preview.selected_primitive = None;
}

impl ProjectTexture {
    fn from_texture(texture: &MaterialTexture) -> Self {
        Self {
            id: texture.id.clone(),
            group_id: texture.group_id.clone(),
            path: texture.path.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectMaterialMeshAssignment {
    pub mesh_id: u64,
    pub object_index: usize,
    pub primitive_index: usize,
}

pub fn save_project_as_dialog(
    atlas: &Atlas,
    settings: &ProjectSettings,
    mesh_scene_state: &MeshSceneState,
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

    ProjectFile::save(&path, atlas, settings, mesh_scene_state)?;

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
    mesh_scene_state: &MeshSceneState,
) -> Result<(), Box<dyn std::error::Error>> {
    ProjectFile::save(path, atlas, settings, mesh_scene_state)
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

fn apply_saved_mesh_parts(scene: &mut import::MeshScene, saved_parts: &[ProjectMeshPart]) {
    for (object_index, object) in scene.objects.iter_mut().enumerate() {
        let mut saved_indices: Vec<usize> = saved_parts
            .iter()
            .filter(|part| part.object_index == object_index)
            .map(|part| part.primitive_index)
            .collect();

        saved_indices.sort_unstable();
        saved_indices.dedup();

        object.primitives = saved_indices
            .into_iter()
            .filter_map(|primitive_index| object.primitives.get(primitive_index).cloned())
            .collect();
    }
}
