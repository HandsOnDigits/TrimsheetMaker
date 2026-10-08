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
        import::{self},
        spawn_mesh_scene,
    },
    project_settings::ProjectSettings,
    transform::{Transform, UvTransform},
    ui::EditorUi,
    viewport::MeshPreview,
};

pub const PROJECT_EXTENSION: &str = "tsm";
pub const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeshPrimitive {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,

    #[serde(default)]
    pub material: Option<import::MeshMaterialRef>,

    #[serde(default)]
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeshObject {
    pub name: String,

    #[serde(default)]
    pub primitives: Vec<ProjectMeshPrimitive>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMesh {
    pub id: u64,
    pub name: String,
    pub path: PathBuf,

    #[serde(default = "default_mesh_visibility")]
    pub visible: bool,

    #[serde(default)]
    pub objects: Vec<ProjectMeshObject>,
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

fn default_mesh_visibility() -> bool {
    true
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

                    objects: mesh
                        .scene
                        .objects
                        .iter()
                        .enumerate()
                        .map(|(object_index, object)| {
                            project_mesh_object_from_runtime(object, object_index, &mesh.parts)
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
                            uv_transform: assignment.uv_transform,
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
                        uv_transform: assignment.uv_transform,
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

    for project_mesh in project_meshes {
        let resolved_path = if project_mesh.path.is_absolute() {
            project_mesh.path.clone()
        } else {
            project_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(&project_mesh.path)
        };

        // The project contains the actual mesh primitives.
        // Do not re-import the source file and do not split again.
        let scene = runtime_mesh_scene_from_project(project_mesh);

        let (root, parts, bounds_center, bounds_radius) = spawn_mesh_scene(
            commands,
            meshes,
            &scene,
            project_mesh.id,
            mesh_preview.material.clone(),
        );

        mesh_scene_state.add_with_id(
            project_mesh.id,
            project_mesh.name.clone(),
            resolved_path,
            scene,
            root,
            parts,
            bounds_center,
            bounds_radius,
        );

        if let Some(mesh) = mesh_scene_state.get_mut(project_mesh.id) {
            mesh.visible = project_mesh.visible;

            for (object_index, object) in project_mesh.objects.iter().enumerate() {
                for (primitive_index, primitive) in object.primitives.iter().enumerate() {
                    if let Some(part) = mesh.parts.iter_mut().find(|part| {
                        part.object_index == object_index && part.primitive_index == primitive_index
                    }) {
                        part.visible = primitive.visible;
                    }
                }
            }
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

    Ok(())
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

    #[serde(default)]
    pub uv_transform: UvTransform,
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

fn project_mesh_object_from_runtime(
    mesh: &crate::mesh::import::MeshObject,
    object_index: usize,
    parts: &[crate::mesh::MeshPartEntry],
) -> ProjectMeshObject {
    ProjectMeshObject {
        name: mesh.name.clone(),

        primitives: mesh
            .primitives
            .iter()
            .enumerate()
            .map(|(primitive_index, primitive)| {
                let visible = parts
                    .iter()
                    .find(|part| {
                        part.object_index == object_index && part.primitive_index == primitive_index
                    })
                    .map(|part| part.visible)
                    .unwrap_or(true);

                ProjectMeshPrimitive {
                    positions: primitive.positions.clone(),
                    normals: primitive.normals.clone(),
                    tangents: primitive.tangents.clone(),
                    uvs: primitive.uvs.clone(),
                    indices: primitive.indices.clone(),
                    material: primitive.material.clone(),
                    visible,
                }
            })
            .collect(),
    }
}

fn runtime_mesh_scene_from_project(project_mesh: &ProjectMesh) -> crate::mesh::import::MeshScene {
    crate::mesh::import::MeshScene {
        objects: project_mesh
            .objects
            .iter()
            .map(|object| crate::mesh::import::MeshObject {
                name: object.name.clone(),

                // ProjectMeshObject currently does not serialize transforms.
                transform: crate::mesh::import::MeshTransform::default(),

                primitives: object
                    .primitives
                    .iter()
                    .map(|primitive| crate::mesh::import::MeshPrimitive {
                        positions: primitive.positions.clone(),
                        normals: primitive.normals.clone(),
                        tangents: primitive.tangents.clone(),
                        uvs: primitive.uvs.clone(),
                        indices: primitive.indices.clone(),
                        material: primitive.material.clone(),
                    })
                    .collect(),
            })
            .collect(),
    }
}
