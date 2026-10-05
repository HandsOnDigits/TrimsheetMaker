pub mod import;
use import::{MeshMaterialRef, MeshScene};

pub mod unwrap;

use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

use crate::viewport::MeshPreviewMaterial;

#[derive(Component)]
pub struct ImportedMesh;

#[derive(Component)]
pub struct ImportedMeshPart {
    pub mesh_id: u64,
    pub part_index: usize,
    pub primitive_index: usize,
    pub source_mesh: Handle<Mesh>,
    pub source_material: Option<MeshMaterialRef>,
}

#[derive(Debug, Clone)]
pub struct MeshEntry {
    pub id: u64,
    pub name: String,
    pub path: std::path::PathBuf,
    pub visible: bool,
    pub scene: import::MeshScene,
    pub root: Option<Entity>,
    pub parts: Vec<MeshPartEntry>,
    pub bounds_center: Vec3,
    pub bounds_radius: f32,
}

#[derive(Debug, Clone)]
pub struct MeshPartEntry {
    pub index: usize,
    pub name: String,
    pub visible: bool,
    pub object_index: usize,
    pub primitive_index: usize,
}

#[derive(Component)]
pub struct ImportedMeshRoot {
    pub id: u64,
}

#[derive(Resource, Default)]
pub struct MeshSceneState {
    pub meshes: Vec<MeshEntry>,
    pub next_id: u64,
}

impl MeshSceneState {
    pub fn add(
        &mut self,
        name: String,
        path: std::path::PathBuf,
        scene: import::MeshScene,
        root: Entity,
        parts: Vec<MeshPartEntry>,
        bounds_center: Vec3,
        bounds_radius: f32,
    ) -> u64 {
        let id = self.next_id;
        self.add_with_id(
            id,
            name,
            path,
            scene,
            root,
            parts,
            bounds_center,
            bounds_radius,
        );
        id
    }

    pub fn remove(&mut self, id: u64) -> Option<MeshEntry> {
        let index = self.meshes.iter().position(|mesh| mesh.id == id)?;
        Some(self.meshes.remove(index))
    }

    pub fn get(&self, id: u64) -> Option<&MeshEntry> {
        self.meshes.iter().find(|mesh| mesh.id == id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut MeshEntry> {
        self.meshes.iter_mut().find(|mesh| mesh.id == id)
    }

    pub fn visible_bounds(&self) -> Option<(Vec3, f32)> {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);

        let mut found = false;

        for mesh in &self.meshes {
            if !mesh.visible {
                continue;
            }

            let radius = mesh.bounds_radius.max(0.0);

            let mesh_min = mesh.bounds_center - Vec3::splat(radius);
            let mesh_max = mesh.bounds_center + Vec3::splat(radius);

            min = min.min(mesh_min);
            max = max.max(mesh_max);

            found = true;
        }

        if !found {
            return None;
        }

        let center = (min + max) * 0.5;
        let radius = (max - center).length().max(1.0);

        Some((center, radius))
    }

    pub fn add_with_id(
        &mut self,
        id: u64,
        name: String,
        path: std::path::PathBuf,
        scene: import::MeshScene,
        root: Entity,
        parts: Vec<MeshPartEntry>,
        bounds_center: Vec3,
        bounds_radius: f32,
    ) {
        self.meshes.push(MeshEntry {
            id,
            name,
            path,
            visible: true,
            scene,
            root: Some(root),
            parts,
            bounds_center,
            bounds_radius,
        });

        self.next_id = self.next_id.max(id.saturating_add(1));
    }
}

#[derive(Resource)]
pub struct MeshLight {
    pub preset: MeshLightPreset,
    pub position: Vec3,
    pub target: Vec3,
    pub follow_camera: bool,
}

impl Default for MeshLight {
    fn default() -> Self {
        Self {
            preset: MeshLightPreset::Studio,
            position: Vec3::new(2.0, 2.0, 2.0),
            target: Vec3::ZERO,
            follow_camera: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshLightPreset {
    Studio,
    Neutral,
    Warm,
    Cool,
    Flat,
    None,
}

impl MeshLightPreset {
    pub const ALL: [Self; 6] = [
        Self::Studio,
        Self::Neutral,
        Self::Warm,
        Self::Cool,
        Self::Flat,
        Self::None,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Studio => "Studio",
            Self::Neutral => "Neutral",
            Self::Warm => "Warm",
            Self::Cool => "Cool",
            Self::Flat => "Flat",
            Self::None => "None",
        }
    }
}

fn create_bevy_mesh(
    meshes: &mut Assets<Mesh>,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    tangents: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
) -> Handle<Mesh> {
    let vertex_count = positions.len();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    if normals.len() == vertex_count {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }
    if tangents.len() == vertex_count {
        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
    }
    if uvs.len() == vertex_count {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }

    if !indices.is_empty() {
        mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    }

    meshes.add(mesh)
}

pub fn spawn_mesh_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    scene: &MeshScene,
    mesh_id: u64,
    material: Handle<MeshPreviewMaterial>,
) -> (Entity, Vec<MeshPartEntry>, Vec3, f32) {
    let mut bounds_min = Vec3::splat(f32::INFINITY);
    let mut bounds_max = Vec3::splat(f32::NEG_INFINITY);

    for object in &scene.objects {
        let transform = Transform {
            translation: Vec3::from(object.transform.translation),
            rotation: Quat::from_xyzw(
                object.transform.rotation[0],
                object.transform.rotation[1],
                object.transform.rotation[2],
                object.transform.rotation[3],
            ),
            scale: Vec3::from(object.transform.scale),
        };

        for primitive in &object.primitives {
            for position in &primitive.positions {
                let world_position = transform.transform_point(Vec3::from(*position));
                bounds_min = bounds_min.min(world_position);
                bounds_max = bounds_max.max(world_position);
            }
        }
    }

    let root = commands
        .spawn((
            Name::new("Imported Mesh"),
            Transform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ImportedMesh,
            ImportedMeshRoot { id: mesh_id },
        ))
        .id();

    let part_count: usize = scene
        .objects
        .iter()
        .map(|object| object.primitives.len())
        .sum();

    let mut parts = Vec::with_capacity(part_count);

    for (object_index, object) in scene.objects.iter().enumerate() {
        for (primitive_index, primitive) in object.primitives.iter().enumerate() {
            let mesh_handle = create_bevy_mesh(
                meshes,
                primitive.positions.clone(),
                primitive.normals.clone(),
                primitive.tangents.clone(),
                primitive.uvs.clone(),
                primitive.indices.clone(),
            );

            let part_index = parts.len();

            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    Name::new(format!("{} Primitive {}", object.name, primitive_index)),
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d::<MeshPreviewMaterial>(material.clone()),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ImportedMesh,
                    ImportedMeshPart {
                        mesh_id,
                        part_index,
                        primitive_index,
                        source_mesh: mesh_handle.clone(),
                        source_material: primitive.material.clone(),
                    },
                ));
            });

            parts.push(MeshPartEntry {
                index: part_index,
                name: object.name.clone(),
                visible: true,
                object_index,
                primitive_index,
            });
        }
    }

    let bounds_center = if bounds_min.x.is_finite() {
        (bounds_min + bounds_max) * 0.5
    } else {
        Vec3::ZERO
    };

    let bounds_radius = if bounds_min.x.is_finite() {
        (bounds_max - bounds_center).length()
    } else {
        1.0
    };

    (root, parts, bounds_center, bounds_radius)
}

pub fn create_imported_bevy_mesh(
    meshes: &mut Assets<Mesh>,
    primitive: &import::MeshPrimitive,
    uvs: Vec<[f32; 2]>,
) -> Handle<Mesh> {
    create_bevy_mesh(
        meshes,
        primitive.positions.clone(),
        primitive.normals.clone(),
        primitive.tangents.clone(),
        uvs,
        primitive.indices.clone(),
    )
}
