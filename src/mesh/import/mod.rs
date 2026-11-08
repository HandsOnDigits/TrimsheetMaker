use bevy::prelude::*;

use mikktspace::generate_tangents;

use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};

pub mod assimp;
pub mod assimp_sys;
pub mod gltf;

#[derive(Debug)]
pub enum MeshImportError {
    UnsupportedFormat(String),
    Io(std::io::Error),
    InvalidData(String),
    Assimp(String),
    Other(String),
}

impl std::fmt::Display for MeshImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFormat(format) => {
                write!(f, "unsupported mesh format: {format}")
            }
            Self::Io(error) => write!(f, "{error}"),
            Self::InvalidData(error) => write!(f, "{error}"),
            Self::Assimp(error) => write!(f, "Assimp error: {error}"),
            Self::Other(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MeshImportError {}

impl From<std::io::Error> for MeshImportError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone)]
pub struct MeshScene {
    pub objects: Vec<MeshObject>,
}

#[derive(Debug, Clone)]
pub struct MeshObject {
    pub name: String,

    /// Local transform from the source scene.
    pub transform: MeshTransform,

    pub primitives: Vec<MeshPrimitive>,
}

#[derive(Debug, Clone)]
pub struct MeshPrimitive {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,

    /// Original UV set 0, when present.
    pub uvs: Vec<[f32; 2]>,

    pub indices: Vec<u32>,

    /// Material name/index from the imported scene.
    pub material: Option<MeshMaterialRef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub struct MeshMaterialRef {
    /// Stable identifier of the imported mesh/material.
    pub id: usize,

    /// Human-readable material name.
    pub name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct MeshTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Default for MeshTransform {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

pub struct MeshImportSettings {
    pub assimp_library: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshBackend {
    Auto,
    Gltf,
    Assimp,
}

impl MeshBackend {
    pub fn from_path(path: &Path) -> Result<Self, MeshImportError> {
        match path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref()
        {
            Some("gltf") | Some("glb") => Ok(Self::Gltf),

            Some(
                "fbx" | "obj" | "dae" | "3ds" | "ply" | "stl",
                // Add other Assimp formats you intend to support.
            ) => Ok(Self::Assimp),

            Some(extension) => Err(MeshImportError::UnsupportedFormat(extension.to_owned())),

            None => Err(MeshImportError::UnsupportedFormat(
                "file has no extension".to_owned(),
            )),
        }
    }
}

impl Default for MeshBackend {
    fn default() -> Self {
        Self::Auto
    }
}

pub fn import(
    path: &Path,
    backend: MeshBackend,
    settings: &MeshImportSettings,
) -> Result<MeshScene, MeshImportError> {
    let backend = match backend {
        MeshBackend::Auto => MeshBackend::from_path(path)?,
        backend => backend,
    };

    let scene = match backend {
        MeshBackend::Auto => unreachable!(),
        MeshBackend::Gltf => gltf::import(path)?,
        MeshBackend::Assimp => assimp::import(path, settings.assimp_library.as_deref())?,
    };

    Ok(scene)
}

struct MikkGeometry<'a> {
    positions: &'a [[f32; 3]],
    normals: &'a [[f32; 3]],
    uvs: &'a [[f32; 2]],
    indices: &'a [u32],
    tangents: Vec<[f32; 4]>,
}

impl<'a> MikkGeometry<'a> {
    fn new(
        positions: &'a [[f32; 3]],
        normals: &'a [[f32; 3]],
        uvs: &'a [[f32; 2]],
        indices: &'a [u32],
    ) -> Self {
        Self {
            positions,
            normals,
            uvs,
            indices,
            tangents: vec![[0.0; 4]; positions.len()],
        }
    }
}

impl mikktspace::Geometry for MikkGeometry<'_> {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        let index = self.indices[face * 3 + vert] as usize;
        self.positions[index]
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        let index = self.indices[face * 3 + vert] as usize;
        self.normals[index]
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let index = self.indices[face * 3 + vert] as usize;
        self.uvs[index]
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        let index = self.indices[face * 3 + vert] as usize;
        self.tangents[index] = tangent;
    }
}

fn generate_mikktspace_tangents(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> Result<Vec<[f32; 4]>, MeshImportError> {
    let mut geometry = MikkGeometry::new(positions, normals, uvs, indices);

    if !generate_tangents(&mut geometry) {
        return Err(MeshImportError::InvalidData(
            "MikkTSpace failed to generate tangents".to_owned(),
        ));
    }

    Ok(geometry.tangents)
}
