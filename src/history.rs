use std::path::PathBuf;

use bevy::prelude::*;

use crate::{
    data::{Atlas, AtlasMaterial, MaterialMeshAssignment, MaterialTexture},
    mesh::{
        MeshSceneState,
        import::{MeshMaterialRef, MeshScene},
    },
    project_settings::ProjectSettings,
    transform::Transform,
};

#[derive(Debug, Clone)]
pub struct EditorSnapshot {
    pub atlas: AtlasSnapshot,
    pub project_settings: ProjectSettings,
    pub meshes: Vec<MeshSnapshot>,
}

impl EditorSnapshot {
    pub fn restore(
        self,
        atlas: &mut Atlas,
        project_settings: &mut ProjectSettings,
        mesh_scene_state: &mut MeshSceneState,
        images: &mut Assets<Image>,
    ) {
        *project_settings = self.project_settings;

        self.atlas.restore(atlas, images);

        mesh_scene_state.restore_document(
            self.meshes
                .into_iter()
                .map(|mesh| (mesh.id, mesh.name, mesh.path, mesh.visible, mesh.scene))
                .collect(),
        );
    }

    pub fn capture(
        atlas: &Atlas,
        project_settings: &ProjectSettings,
        mesh_scene_state: &MeshSceneState,
    ) -> Self {
        Self {
            atlas: AtlasSnapshot {
                width: atlas.width,
                height: atlas.height,
                padding: atlas.padding,
                materials: atlas
                    .materials
                    .iter()
                    .map(AtlasMaterialSnapshot::from_material)
                    .collect(),
            },

            project_settings: project_settings.clone(),

            meshes: mesh_scene_state
                .meshes
                .iter()
                .map(|mesh| MeshSnapshot {
                    id: mesh.id,
                    name: mesh.name.clone(),
                    path: mesh.path.clone(),
                    visible: mesh.visible,
                    scene: mesh.scene.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtlasSnapshot {
    pub width: u32,
    pub height: u32,
    pub padding: u32,
    pub materials: Vec<AtlasMaterialSnapshot>,
}

impl AtlasSnapshot {
    pub fn restore(self, atlas: &mut Atlas, images: &mut Assets<Image>) {
        atlas.width = self.width;
        atlas.height = self.height;
        atlas.padding = self.padding;

        atlas.materials = self
            .materials
            .into_iter()
            .map(AtlasMaterialSnapshot::restore)
            .collect();

        atlas.images = Default::default();

        atlas.load_textures(images);

        atlas.mark_uv_dirty();
        atlas.mark_image_dirty();
        atlas.mark_bake_dirty();
    }
}

#[derive(Debug, Clone)]
pub struct AtlasMaterialSnapshot {
    pub index: usize,
    pub name: String,
    pub material_ref: Option<MeshMaterialRef>,
    pub textures: Vec<MaterialTextureSnapshot>,
    pub transform: Transform,
    pub mesh_assignments: Vec<MaterialMeshAssignmentSnapshot>,
}

impl AtlasMaterialSnapshot {
    fn restore(self) -> AtlasMaterial {
        AtlasMaterial {
            index: self.index,
            name: self.name,
            material_ref: self.material_ref,
            textures: self
                .textures
                .into_iter()
                .map(MaterialTextureSnapshot::restore)
                .collect(),
            transform: self.transform,
            mesh_assignments: self
                .mesh_assignments
                .into_iter()
                .map(|assignment| MaterialMeshAssignment {
                    mesh_id: assignment.mesh_id,
                    object_index: assignment.object_index,
                    primitive_index: assignment.primitive_index,
                })
                .collect(),
        }
    }
}

impl AtlasMaterialSnapshot {
    fn from_material(material: &AtlasMaterial) -> Self {
        Self {
            index: material.index,
            name: material.name.clone(),
            material_ref: material.material_ref.clone(),

            textures: material
                .textures
                .iter()
                .map(|texture| MaterialTextureSnapshot {
                    id: texture.id.clone(),
                    path: texture.path.clone(),
                    group_id: texture.group_id.clone(),
                })
                .collect(),

            transform: material.transform,

            mesh_assignments: material
                .mesh_assignments
                .iter()
                .map(|assignment| MaterialMeshAssignmentSnapshot {
                    mesh_id: assignment.mesh_id,
                    object_index: assignment.object_index,
                    primitive_index: assignment.primitive_index,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialTextureSnapshot {
    pub id: String,
    pub path: PathBuf,
    pub group_id: Option<String>,
}

impl MaterialTextureSnapshot {
    fn restore(self) -> MaterialTexture {
        MaterialTexture {
            id: self.id,
            path: self.path,
            group_id: self.group_id,
            image: None,
            egui_texture: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialMeshAssignmentSnapshot {
    pub mesh_id: u64,
    pub object_index: usize,
    pub primitive_index: usize,
}

#[derive(Debug, Clone)]
pub struct MeshSnapshot {
    pub id: u64,
    pub name: String,
    pub path: PathBuf,
    pub visible: bool,
    pub scene: MeshScene,
}

#[derive(Resource, Default)]
pub struct EditorHistory {
    undo: Vec<EditorSnapshot>,
    redo: Vec<EditorSnapshot>,
}

impl EditorHistory {
    pub fn begin(
        &self,
        atlas: &Atlas,
        project_settings: &ProjectSettings,
        mesh_scene_state: &MeshSceneState,
    ) -> EditorSnapshot {
        EditorSnapshot::capture(atlas, project_settings, mesh_scene_state)
    }

    pub fn commit(&mut self, before: EditorSnapshot, changed: bool) {
        if changed {
            self.undo.push(before);
            self.redo.clear();
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn push(&mut self, snapshot: EditorSnapshot) {
        self.undo.push(snapshot);
        self.redo.clear();
    }

    pub fn undo(
        &mut self,
        atlas: &mut Atlas,
        project_settings: &mut ProjectSettings,
        mesh_scene_state: &mut MeshSceneState,
        images: &mut Assets<Image>,
    ) -> bool {
        let Some(previous) = self.undo.pop() else {
            return false;
        };

        let current = EditorSnapshot::capture(atlas, project_settings, mesh_scene_state);

        self.redo.push(current);

        previous.restore(atlas, project_settings, mesh_scene_state, images);

        true
    }

    pub fn redo(
        &mut self,
        atlas: &mut Atlas,
        project_settings: &mut ProjectSettings,
        mesh_scene_state: &mut MeshSceneState,
        images: &mut Assets<Image>,
    ) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };

        let current = EditorSnapshot::capture(atlas, project_settings, mesh_scene_state);

        self.undo.push(current);

        next.restore(atlas, project_settings, mesh_scene_state, images);

        true
    }
}
