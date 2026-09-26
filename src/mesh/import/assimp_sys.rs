use std::ffi::c_char;

pub type AiReal = f32;

pub const AI_MAX_NUMBER_OF_COLOR_SETS: usize = 8;
pub const AI_MAX_NUMBER_OF_TEXTURECOORDS: usize = 8;

pub const AI_PROCESS_CALC_TANGENT_SPACE: u32 = 0x1;
pub const AI_PROCESS_JOIN_IDENTICAL_VERTICES: u32 = 0x2;
pub const AI_PROCESS_TRIANGULATE: u32 = 0x8;
pub const AI_PROCESS_SORT_BY_PTYPE: u32 = 0x8000;
pub const AI_PROCESS_GEN_SMOOTH_NORMALS: u32 = 0x40;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AiString {
    pub length: u32,
    pub data: [c_char; 1024],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AiVector3D {
    pub x: AiReal,
    pub y: AiReal,
    pub z: AiReal,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AiMatrix4x4 {
    pub a1: AiReal,
    pub a2: AiReal,
    pub a3: AiReal,
    pub a4: AiReal,

    pub b1: AiReal,
    pub b2: AiReal,
    pub b3: AiReal,
    pub b4: AiReal,

    pub c1: AiReal,
    pub c2: AiReal,
    pub c3: AiReal,
    pub c4: AiReal,

    pub d1: AiReal,
    pub d2: AiReal,
    pub d3: AiReal,
    pub d4: AiReal,
}

#[repr(C)]
#[derive(Debug)]
pub struct AiFace {
    pub num_indices: u32,
    pub indices: *mut u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct AiMesh {
    pub primitive_types: u32,
    pub num_vertices: u32,
    pub num_faces: u32,

    pub vertices: *mut AiVector3D,
    pub normals: *mut AiVector3D,
    pub tangents: *mut AiVector3D,
    pub bitangents: *mut AiVector3D,

    pub colors: [*mut std::ffi::c_void; AI_MAX_NUMBER_OF_COLOR_SETS],
    pub texture_coords: [*mut AiVector3D; AI_MAX_NUMBER_OF_TEXTURECOORDS],

    pub num_uv_components: [u32; AI_MAX_NUMBER_OF_TEXTURECOORDS],

    pub faces: *mut AiFace,

    pub num_bones: u32,
    pub bones: *mut std::ffi::c_void,

    pub material_index: u32,

    pub name: AiString,

    pub num_anim_meshes: u32,
    pub anim_meshes: *mut std::ffi::c_void,

    pub method: i32,

    // aiAABB comes after this. We don't need it for our current conversion,
    // but it MUST remain represented so that the following field offsets
    // would remain correct if we add later fields.
    pub aabb: AiAabb,

    pub texture_coords_names: *mut *mut AiString,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AiAabb {
    pub min: AiVector3D,
    pub max: AiVector3D,
}

#[repr(C)]
#[derive(Debug)]
pub struct AiNode {
    pub name: AiString,
    pub transformation: AiMatrix4x4,

    pub parent: *mut AiNode,

    pub num_children: u32,
    pub children: *mut *mut AiNode,

    pub num_meshes: u32,
    pub meshes: *mut u32,

    pub metadata: *mut std::ffi::c_void,
}

#[repr(C)]
#[derive(Debug)]
pub struct AiScene {
    pub flags: u32,

    pub root_node: *mut AiNode,

    pub num_meshes: u32,
    pub meshes: *mut *mut AiMesh,

    pub num_materials: u32,
    pub materials: *mut *mut std::ffi::c_void,

    pub num_animations: u32,
    pub animations: *mut std::ffi::c_void,

    pub num_textures: u32,
    pub textures: *mut std::ffi::c_void,

    pub num_lights: u32,
    pub lights: *mut std::ffi::c_void,

    pub num_cameras: u32,
    pub cameras: *mut std::ffi::c_void,

    pub metadata: *mut std::ffi::c_void,

    pub name: AiString,

    pub num_skeletons: u32,
    pub skeletons: *mut std::ffi::c_void,

    pub private: *mut std::ffi::c_void,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    #[test]
    fn abi_layout_is_expected() {
        assert_eq!(size_of::<AiVector3D>(), 12);
        assert_eq!(size_of::<AiMatrix4x4>(), 64);
        assert_eq!(size_of::<AiFace>(), 16);
        assert_eq!(size_of::<AiString>(), 1028);

        assert_eq!(align_of::<AiVector3D>(), 4);
        assert_eq!(align_of::<AiMatrix4x4>(), 4);

        assert_eq!(offset_of!(AiScene, flags), 0);
        assert_eq!(offset_of!(AiScene, root_node), size_of::<usize>());
        assert_eq!(offset_of!(AiScene, num_meshes), size_of::<usize>() * 2);
        assert_eq!(offset_of!(AiScene, meshes), size_of::<usize>() * 2 + 4);
    }
}
