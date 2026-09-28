pub mod import;
use import::MeshScene;

mod regions;
pub mod unwrap;
pub mod viewport;

use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

#[derive(Component)]
pub struct ImportedMesh;

#[derive(Component)]
pub struct ImportedMeshPart {
    pub mesh_id: u64,
    pub part_index: usize,
    pub primitive_index: Option<usize>,
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
    pub entity: Option<Entity>,
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
        self.next_id += 1;

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
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));

    meshes.add(mesh)
}

pub fn spawn_mesh_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    scene: &MeshScene,
    mesh_id: u64,
) -> (Entity, Vec<MeshPartEntry>, Vec3, f32) {
    let mut bounds_min = Vec3::splat(f32::INFINITY);
    let mut bounds_max = Vec3::splat(f32::NEG_INFINITY);

    for object in &scene.objects {
        let translation = Vec3::from(object.transform.translation);

        let rotation = Quat::from_xyzw(
            object.transform.rotation[0],
            object.transform.rotation[1],
            object.transform.rotation[2],
            object.transform.rotation[3],
        );

        let scale = Vec3::from(object.transform.scale);

        let transform = Transform {
            translation,
            rotation,
            scale,
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

    let mut parts = Vec::with_capacity(scene.objects.len());

    for (part_index, object) in scene.objects.iter().enumerate() {
        let object_entity = commands
            .spawn((
                Name::new(object.name.clone()),
                Transform::from_translation(Vec3::from(object.transform.translation))
                    .with_rotation(Quat::from_xyzw(
                        object.transform.rotation[0],
                        object.transform.rotation[1],
                        object.transform.rotation[2],
                        object.transform.rotation[3],
                    ))
                    .with_scale(Vec3::from(object.transform.scale)),
                Visibility::Visible,
                InheritedVisibility::default(),
                ImportedMesh,
                ImportedMeshPart {
                    mesh_id,
                    part_index,
                    primitive_index: None,
                },
            ))
            .id();

        commands.entity(root).add_child(object_entity);

        for (primitive_index, primitive) in object.primitives.iter().enumerate() {
            let mesh_handle = create_bevy_mesh(
                meshes,
                primitive.positions.clone(),
                primitive.normals.clone(),
                primitive.tangents.clone(),
                primitive.uvs.clone(),
                primitive.indices.clone(),
            );

            let material_handle = materials.add(StandardMaterial {
                base_color: Color::srgb(0.7, 0.7, 0.7),
                perceptual_roughness: 0.7,
                ..default()
            });

            commands.entity(object_entity).with_children(|parent| {
                parent.spawn((
                    Name::new(format!("{} Primitive {}", object.name, primitive_index)),
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ImportedMesh,
                    ImportedMeshPart {
                        mesh_id,
                        part_index,
                        primitive_index: Some(primitive_index),
                    },
                ));
            });
        }

        parts.push(MeshPartEntry {
            index: part_index,
            name: object.name.clone(),
            visible: true,
            entity: Some(object_entity),
        });
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
