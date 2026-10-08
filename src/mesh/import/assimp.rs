use bevy::prelude::Vec4;

use std::{
    ffi::{CStr, CString, c_char},
    path::{Path, PathBuf},
};

use libloading::Library;

use crate::{
    mesh::{
        import::assimp_sys::{
            AI_PROCESS_CALC_TANGENT_SPACE, AI_PROCESS_GEN_SMOOTH_NORMALS,
            AI_PROCESS_JOIN_IDENTICAL_VERTICES, AI_PROCESS_TRIANGULATE, AiScene,
        },
        import::{
            MeshImportError, MeshMaterialRef, MeshObject, MeshPrimitive, MeshScene, MeshTransform,
        },
    },
    transform::UvTransform,
};

#[derive(Debug)]
pub enum AssimpError {
    LoadLibrary {
        path: PathBuf,
        error: libloading::Error,
    },
    MissingSymbol(&'static str),
}

impl std::fmt::Display for AssimpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadLibrary { path, error } => {
                write!(
                    f,
                    "failed to load Assimp library '{}': {error}",
                    path.display()
                )
            }
            Self::MissingSymbol(symbol) => {
                write!(f, "Assimp library is missing symbol '{symbol}'")
            }
        }
    }
}

pub struct AssimpLibrary {
    _library: Library,
    path: PathBuf,

    get_version_major: unsafe extern "C" fn() -> u32,
    get_version_minor: unsafe extern "C" fn() -> u32,
    get_version_revision: unsafe extern "C" fn() -> u32,

    ai_import_file: unsafe extern "C" fn(*const c_char, u32) -> *const AiScene,
    ai_release_import: unsafe extern "C" fn(*const AiScene),
    ai_get_error_string: unsafe extern "C" fn() -> *const c_char,

    ai_get_material_string: unsafe extern "C" fn(
        *const std::ffi::c_void,
        *const c_char,
        u32,
        u32,
        *mut crate::mesh::import::assimp_sys::AiString,
    ) -> i32,
}

impl AssimpLibrary {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AssimpError> {
        let path = path.as_ref().to_path_buf();

        let library = unsafe { Library::new(&path) }.map_err(|error| AssimpError::LoadLibrary {
            path: path.clone(),
            error,
        })?;

        let get_version_major = unsafe {
            *library
                .get::<unsafe extern "C" fn() -> u32>(b"aiGetVersionMajor\0")
                .map_err(|_| AssimpError::MissingSymbol("aiGetVersionMajor"))?
        };

        let get_version_minor = unsafe {
            *library
                .get::<unsafe extern "C" fn() -> u32>(b"aiGetVersionMinor\0")
                .map_err(|_| AssimpError::MissingSymbol("aiGetVersionMinor"))?
        };

        let get_version_revision = unsafe {
            *library
                .get::<unsafe extern "C" fn() -> u32>(b"aiGetVersionRevision\0")
                .map_err(|_| AssimpError::MissingSymbol("aiGetVersionRevision"))?
        };

        let ai_import_file = unsafe {
            *library
                .get::<unsafe extern "C" fn(*const c_char, u32) -> *const AiScene>(
                    b"aiImportFile\0",
                )
                .map_err(|_| AssimpError::MissingSymbol("aiImportFile"))?
        };

        let ai_release_import = unsafe {
            *library
                .get::<unsafe extern "C" fn(*const AiScene)>(b"aiReleaseImport\0")
                .map_err(|_| AssimpError::MissingSymbol("aiReleaseImport"))?
        };

        let ai_get_error_string = unsafe {
            *library
                .get::<unsafe extern "C" fn() -> *const c_char>(b"aiGetErrorString\0")
                .map_err(|_| AssimpError::MissingSymbol("aiGetErrorString"))?
        };

        let ai_get_material_string = unsafe {
            *library
                .get::<unsafe extern "C" fn(
                    *const std::ffi::c_void,
                    *const c_char,
                    u32,
                    u32,
                    *mut crate::mesh::import::assimp_sys::AiString,
                ) -> i32>(b"aiGetMaterialString\0")
                .map_err(|_| AssimpError::MissingSymbol("aiGetMaterialString"))?
        };

        Ok(Self {
            _library: library,
            path,
            get_version_major,
            get_version_minor,
            get_version_revision,
            ai_import_file,
            ai_release_import,
            ai_get_error_string,
            ai_get_material_string,
        })
    }

    #[allow(unused)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[allow(unused)]
    pub fn version(&self) -> (u32, u32, u32) {
        unsafe {
            (
                (self.get_version_major)(),
                (self.get_version_minor)(),
                (self.get_version_revision)(),
            )
        }
    }

    #[cfg(test)]
    pub fn test_import(&self, path: &Path) -> Result<(), String> {
        let path = CString::new(path.to_string_lossy().as_bytes())
            .map_err(|_| "mesh path contains an embedded null byte".to_owned())?;

        // Start with no post-processing flags.
        let scene = unsafe { (self.ai_import_file)(path.as_ptr(), 0) };

        if scene.is_null() {
            let error = unsafe { (self.ai_get_error_string)() };

            if error.is_null() {
                return Err("Assimp import failed with no error message".to_owned());
            }

            let error = unsafe { std::ffi::CStr::from_ptr(error) };

            return Err(error.to_string_lossy().into_owned());
        }

        let scene_ref = unsafe { &*scene };

        println!("Assimp successfully imported the file.");
        println!("  flags:       0x{:X}", scene_ref.flags);
        println!("  meshes:      {}", scene_ref.num_meshes);
        println!("  materials:   {}", scene_ref.num_materials);
        println!("  animations:  {}", scene_ref.num_animations);
        println!("  textures:    {}", scene_ref.num_textures);
        println!("  root node:   {:p}", scene_ref.root_node);

        if !scene_ref.root_node.is_null() {
            let root = unsafe { &*scene_ref.root_node };

            println!("  root name:   {}", ai_string_to_string(&root.name));
            println!("  root meshes: {}", root.num_meshes);
            println!("  root children: {}", root.num_children);
        }

        unsafe {
            (self.ai_release_import)(scene);
        }

        Ok(())
    }

    pub fn import_scene(&self, path: &Path) -> Result<*const AiScene, String> {
        let path = CString::new(path.to_string_lossy().as_bytes())
            .map_err(|_| "mesh path contains an embedded null byte".to_owned())?;

        let flags = AI_PROCESS_TRIANGULATE
            | AI_PROCESS_JOIN_IDENTICAL_VERTICES
            | AI_PROCESS_GEN_SMOOTH_NORMALS
            | AI_PROCESS_CALC_TANGENT_SPACE;

        let scene = unsafe { (self.ai_import_file)(path.as_ptr(), flags) };

        if scene.is_null() {
            let error = unsafe { (self.ai_get_error_string)() };

            if error.is_null() {
                return Err("Assimp import failed with no error message".to_owned());
            }

            let error = unsafe { CStr::from_ptr(error) };

            return Err(error.to_string_lossy().into_owned());
        }

        Ok(scene)
    }

    pub unsafe fn release_scene(&self, scene: *const AiScene) {
        unsafe {
            (self.ai_release_import)(scene);
        }
    }

    fn material_name(&self, material: *const std::ffi::c_void) -> String {
        if material.is_null() {
            return String::new();
        }

        let key = CString::new("?mat.name").expect("static material key contains no null");

        let mut name = crate::mesh::import::assimp_sys::AiString {
            length: 0,
            data: [0; 1024],
        };

        let result =
            unsafe { (self.ai_get_material_string)(material, key.as_ptr(), 0, 0, &mut name) };

        // AI_SUCCESS == 0
        if result != 0 {
            return String::new();
        }

        ai_string_to_string(&name)
    }
}

pub fn import(path: &Path, library_path: Option<&Path>) -> Result<MeshScene, MeshImportError> {
    let library_path = library_path.ok_or_else(|| {
        MeshImportError::Assimp("no Assimp library has been configured".to_owned())
    })?;

    let library = AssimpLibrary::load(library_path)
        .map_err(|error| MeshImportError::Assimp(error.to_string()))?;

    let scene = library
        .import_scene(path)
        .map_err(MeshImportError::Assimp)?;

    let scene = ImportedScene {
        library: &library,
        scene,
    };

    unsafe { convert_scene(scene.scene, &library) }.map_err(MeshImportError::Assimp)
}

unsafe fn convert_scene(
    scene: *const AiScene,
    library: &AssimpLibrary,
) -> Result<MeshScene, String> {
    if scene.is_null() {
        return Err("Assimp returned a null scene".to_owned());
    }

    let scene_ref = unsafe { &*scene };

    if scene_ref.root_node.is_null() {
        return Err("Assimp scene has no root node".to_owned());
    }

    let mut objects = Vec::new();

    unsafe {
        collect_node_objects(scene_ref.root_node, scene_ref, library, &mut objects)?;
    }

    Ok(MeshScene { objects })
}

unsafe fn collect_node_objects(
    node: *const crate::mesh::import::assimp_sys::AiNode,
    scene: &crate::mesh::import::assimp_sys::AiScene,
    library: &AssimpLibrary,
    objects: &mut Vec<MeshObject>,
) -> Result<(), String> {
    if node.is_null() {
        return Err("Assimp scene contains a null node".to_owned());
    }

    let node = unsafe { &*node };

    let name = ai_string_to_string(&node.name);

    let transform = mesh_transform_from_assimp(&node.transformation);

    let mut primitives = Vec::new();

    if node.num_meshes > 0 {
        if node.meshes.is_null() {
            return Err(format!(
                "Assimp node '{}' has meshes but its mesh array is null",
                name
            ));
        }
    }

    for mesh_index in 0..node.num_meshes {
        let mesh_index = unsafe { *node.meshes.add(mesh_index as usize) };

        if mesh_index >= scene.num_meshes {
            return Err(format!(
                "Assimp node '{}' references invalid mesh index {}",
                name, mesh_index
            ));
        }

        if scene.meshes.is_null() {
            return Err("Assimp scene has meshes but its mesh array is null".to_owned());
        }

        let mesh_ptr = unsafe { *scene.meshes.add(mesh_index as usize) };

        if mesh_ptr.is_null() {
            return Err(format!("Assimp mesh {} is null", mesh_index));
        }

        let mesh = unsafe { &*mesh_ptr };

        unsafe {
            if let Some(primitive) = convert_mesh(mesh, scene, library)? {
                primitives.push(primitive);
            }
        }
    }

    if !primitives.is_empty() {
        objects.push(MeshObject {
            name: name.clone(),
            transform,
            primitives,
        });
    }

    if node.num_children > 0 && node.children.is_null() {
        return Err(format!(
            "Assimp node '{}' has children but its child array is null",
            name
        ));
    }

    for child_index in 0..node.num_children {
        let child = unsafe { *node.children.add(child_index as usize) };

        unsafe {
            collect_node_objects(child, scene, library, objects)?;
        }
    }

    Ok(())
}

unsafe fn convert_mesh(
    mesh: &crate::mesh::import::assimp_sys::AiMesh,
    scene: &crate::mesh::import::assimp_sys::AiScene,
    library: &AssimpLibrary,
) -> Result<Option<MeshPrimitive>, String> {
    if mesh.num_vertices == 0 {
        return Ok(None);
    }

    if mesh.vertices.is_null() {
        return Err("Assimp mesh has vertices but its vertex array is null".to_owned());
    }

    if mesh.num_faces > 0 && mesh.faces.is_null() {
        return Err("Assimp mesh has faces but its face array is null".to_owned());
    }

    let vertex_count = mesh.num_vertices as usize;

    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut tangents = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);

    for index in 0..vertex_count {
        let vertex = unsafe { &*mesh.vertices.add(index) };

        positions.push([vertex.x, vertex.y, vertex.z]);

        if !mesh.normals.is_null() {
            let normal = unsafe { &*mesh.normals.add(index) };

            normals.push([normal.x, normal.y, normal.z]);
        } else {
            normals.push([0.0, 0.0, 0.0]);
        }

        if !mesh.tangents.is_null() {
            let tangent = unsafe { &*mesh.tangents.add(index) };

            tangents.push([tangent.x, tangent.y, tangent.z, 1.0]);
        } else {
            tangents.push([0.0, 0.0, 0.0, 1.0]);
        }

        if !mesh.texture_coords[0].is_null() {
            let uv = unsafe { &*mesh.texture_coords[0].add(index) };

            uvs.push([uv.x, uv.y]);
        } else {
            uvs.push([0.0, 0.0]);
        }
    }

    let mut indices = Vec::new();

    for face_index in 0..mesh.num_faces as usize {
        let face = unsafe { &*mesh.faces.add(face_index) };

        if face.num_indices != 3 {
            return Err(format!(
                "Assimp returned a non-triangle face with {} indices after triangulation",
                face.num_indices
            ));
        }

        if face.indices.is_null() {
            return Err(format!(
                "Assimp mesh contains a face with {} indices but no index array",
                face.num_indices
            ));
        }

        for index in 0..3 {
            let vertex_index = unsafe { *face.indices.add(index) };

            if vertex_index >= mesh.num_vertices {
                return Err(format!(
                    "Assimp face references invalid vertex index {}",
                    vertex_index
                ));
            }

            indices.push(vertex_index);
        }
    }

    let material_id = mesh.material_index as usize;

    if material_id >= scene.num_materials as usize {
        return Err(format!(
            "Assimp mesh references invalid material index {}",
            material_id
        ));
    }

    if scene.materials.is_null() {
        return Err("Assimp scene has materials but its material array is null".to_owned());
    }

    let material_ptr = unsafe { *scene.materials.add(material_id) };

    let material_name = library.material_name(material_ptr);

    let material = Some(MeshMaterialRef {
        id: material_id,
        name: if material_name.is_empty() {
            format!("Material {}", material_id)
        } else {
            material_name
        },
    });

    Ok(Some(MeshPrimitive {
        positions,
        normals,
        tangents,
        uvs,
        indices,
        material,
        uv_transform: UvTransform::default(),
    }))
}

fn ai_string_to_string(string: &crate::mesh::import::assimp_sys::AiString) -> String {
    let length = (string.length as usize).min(string.data.len());

    let bytes = string.data[..length]
        .iter()
        .map(|&value| value as u8)
        .collect::<Vec<_>>();

    String::from_utf8_lossy(&bytes).into_owned()
}

fn mesh_transform_from_assimp(
    matrix: &crate::mesh::import::assimp_sys::AiMatrix4x4,
) -> MeshTransform {
    let matrix = bevy::math::Mat4::from_cols(
        Vec4::new(matrix.a1, matrix.b1, matrix.c1, matrix.d1),
        Vec4::new(matrix.a2, matrix.b2, matrix.c2, matrix.d2),
        Vec4::new(matrix.a3, matrix.b3, matrix.c3, matrix.d3),
        Vec4::new(matrix.a4, matrix.b4, matrix.c4, matrix.d4),
    );

    let (scale, rotation, translation) = matrix.to_scale_rotation_translation();

    MeshTransform {
        translation: translation.to_array(),
        rotation: [rotation.x, rotation.y, rotation.z, rotation.w],
        scale: scale.to_array(),
    }
}

struct ImportedScene<'a> {
    library: &'a AssimpLibrary,
    scene: *const AiScene,
}

impl Drop for ImportedScene<'_> {
    fn drop(&mut self) {
        unsafe {
            self.library.release_scene(self.scene);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_and_import_assimp_library() {
        let library_path = std::env::var("ASSIMP_LIBRARY")
            .expect("ASSIMP_LIBRARY environment variable is not set");

        let mesh_path = std::env::var("ASSIMP_TEST_MODEL")
            .expect("ASSIMP_TEST_MODEL environment variable is not set");

        let library = AssimpLibrary::load(&library_path).expect("failed to load Assimp library");

        let version = library.version();

        println!("Assimp {}.{}.{}", version.0, version.1, version.2);

        println!("Loaded from: {}", library.path().display());

        library
            .test_import(Path::new(&mesh_path))
            .expect("Assimp failed to import test model");
    }

    #[test]
    fn import_assimp_scene() {
        let library_path = std::env::var("ASSIMP_LIBRARY")
            .expect("ASSIMP_LIBRARY environment variable is not set");

        let mesh_path = std::env::var("ASSIMP_TEST_MODEL")
            .expect("ASSIMP_TEST_MODEL environment variable is not set");

        let scene = super::import(Path::new(&mesh_path), Some(Path::new(&library_path)))
            .expect("failed to import Assimp scene");

        println!("Imported {} mesh objects", scene.objects.len());

        for object in &scene.objects {
            println!(
                "Object '{}' - {} primitives",
                object.name,
                object.primitives.len()
            );

            for (index, primitive) in object.primitives.iter().enumerate() {
                println!(
                    "  Primitive {}: {} vertices, {} indices, material {:?}",
                    index,
                    primitive.positions.len(),
                    primitive.indices.len(),
                    primitive.material
                );
            }
        }

        assert!(!scene.objects.is_empty());

        for object in &scene.objects {
            for primitive in &object.primitives {
                assert_eq!(primitive.positions.len(), primitive.normals.len());
                assert_eq!(primitive.positions.len(), primitive.tangents.len());
                assert_eq!(primitive.positions.len(), primitive.uvs.len());
                assert_eq!(primitive.indices.len() % 3, 0);
                assert_eq!(
                    primitive
                        .material
                        .as_ref()
                        .map(|material| material.name.as_str()),
                    Some("DefaultMaterial")
                );
                assert!(
                    primitive
                        .material
                        .as_ref()
                        .is_some_and(|material| !material.name.is_empty())
                );
            }
        }
    }
}
