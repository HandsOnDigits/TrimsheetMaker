use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub struct MeshMaterialRef {
    /// Identifier belonging to the imported scene.
    ///
    /// This is deliberately NOT an AtlasMaterial index.
    pub id: usize,

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

pub enum MeshBackend {
    Gltf,
    Assimp,
}
