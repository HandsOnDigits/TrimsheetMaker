use anyhow::{Result, anyhow, ensure};
use xatlas_rs_v2::{ChartOptions, IndexData, MeshData, MeshDecl, PackOptions, Xatlas};

use crate::mesh::{
    import::{MeshMaterialRef, MeshPrimitive, MeshScene, MeshTransform},
    viewport::MeshPreview,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeshUvMode {
    #[default]
    Imported,
    Unwrapped,
}

#[derive(Debug, Clone)]
pub struct UnwrapInput {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct UnwrappedMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
    pub vertex_xrefs: Vec<u32>,

    pub atlas_indices: Vec<i32>,
    pub chart_indices: Vec<i32>,

    pub atlas_count: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub chart_count: u32,
}

pub struct UnwrappedMeshScene {
    pub objects: Vec<UnwrappedMeshObject>,
}

impl UnwrappedMeshScene {
    pub fn primitive(
        &self,
        object_index: usize,
        primitive_index: usize,
    ) -> Option<&UnwrappedPrimitive> {
        self.objects
            .get(object_index)?
            .primitives
            .get(primitive_index)
    }
}

pub struct UnwrappedMeshObject {
    pub name: String,
    pub transform: MeshTransform,
    pub primitives: Vec<UnwrappedPrimitive>,
}

pub fn unwrap_mesh(input: &UnwrapInput) -> Result<UnwrappedMesh> {
    ensure!(!input.positions.is_empty(), "Cannot unwrap an empty mesh");

    ensure!(
        !input.indices.is_empty(),
        "Cannot unwrap a mesh without indices"
    );

    ensure!(
        input.indices.len().is_multiple_of(3),
        "Mesh indices must contain complete triangles"
    );

    ensure!(
        input
            .indices
            .iter()
            .all(|&index| { (index as usize) < input.positions.len() }),
        "Mesh contains an index outside the vertex array"
    );

    let position_data: Vec<f32> = input
        .positions
        .iter()
        .flat_map(|position| position.iter().copied())
        .collect();

    let mesh_decl = MeshDecl {
        vertex_position_data: MeshData::Contiguous(&position_data),
        index_data: Some(IndexData::U32(&input.indices)),
        face_count: (input.indices.len() / 3) as u32,
        ..Default::default()
    };

    let mut atlas = Xatlas::new();

    atlas
        .add_mesh(&mesh_decl)
        .map_err(|error| anyhow!("Failed to add mesh to xatlas: {error:?}"))?;

    let chart_options = ChartOptions::default();
    let pack_options = PackOptions::default();

    atlas.generate(&chart_options, &pack_options);

    ensure!(atlas.mesh_count() > 0, "xatlas generated no output meshes");

    let meshes = atlas.meshes();

    ensure!(!meshes.is_empty(), "xatlas returned no generated meshes");

    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new();
    let mut vertex_xrefs = Vec::new();
    let mut atlas_indices = Vec::new();
    let mut chart_indices = Vec::new();

    let mut vertex_offset = 0_u32;

    for mesh in meshes {
        for vertex in &mesh.vertex_array {
            let xref = vertex.xref as usize;

            ensure!(
                xref < input.positions.len(),
                "xatlas returned invalid vertex xref {}",
                vertex.xref
            );

            positions.push(input.positions[xref]);

            let uv = [
                if atlas.width() > 0 {
                    vertex.uv[0] / atlas.width() as f32
                } else {
                    0.0
                },
                if atlas.height() > 0 {
                    vertex.uv[1] / atlas.height() as f32
                } else {
                    0.0
                },
            ];

            uvs.push(uv);

            vertex_xrefs.push(vertex.xref);
            atlas_indices.push(vertex.atlas_index);
            chart_indices.push(vertex.chart_index);
        }

        for &index in mesh.index_array {
            indices.push(index + vertex_offset);
        }

        vertex_offset += mesh.vertex_array.len() as u32;
    }

    Ok(UnwrappedMesh {
        positions,
        indices,
        uvs,
        vertex_xrefs,
        atlas_indices,
        chart_indices,
        atlas_count: atlas.atlas_count(),
        atlas_width: atlas.width(),
        atlas_height: atlas.height(),
        chart_count: atlas.chart_count(),
    })
}

#[derive(Debug, Clone)]
pub struct UnwrappedPrimitive {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,

    pub material: Option<MeshMaterialRef>,

    /// Maps each generated vertex back to the source MeshPrimitive vertex.
    pub vertex_xrefs: Vec<u32>,

    /// xatlas atlas index for each generated vertex.
    pub atlas_indices: Vec<i32>,

    /// xatlas chart index for each generated vertex.
    pub chart_indices: Vec<i32>,

    pub atlas_count: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub chart_count: u32,
}

pub fn unwrap_primitive(primitive: &MeshPrimitive) -> Result<UnwrappedPrimitive> {
    ensure!(
        !primitive.positions.is_empty(),
        "Cannot unwrap an empty primitive"
    );

    ensure!(
        primitive.positions.len() == primitive.normals.len(),
        "Position/normal vertex count mismatch"
    );

    ensure!(
        primitive.positions.len() == primitive.tangents.len(),
        "Position/tangent vertex count mismatch"
    );

    ensure!(
        primitive.positions.len() == primitive.uvs.len(),
        "Position/UV vertex count mismatch"
    );

    let unwrapped = unwrap_mesh(&UnwrapInput {
        positions: primitive.positions.clone(),
        indices: primitive.indices.clone(),
    })?;

    let mut normals = Vec::with_capacity(unwrapped.positions.len());
    let mut tangents = Vec::with_capacity(unwrapped.positions.len());

    for &xref in &unwrapped.vertex_xrefs {
        let source_index = xref as usize;

        ensure!(
            source_index < primitive.normals.len(),
            "xatlas returned invalid normal xref {}",
            xref
        );

        ensure!(
            source_index < primitive.tangents.len(),
            "xatlas returned invalid tangent xref {}",
            xref
        );

        normals.push(primitive.normals[source_index]);
        tangents.push(primitive.tangents[source_index]);
    }

    Ok(UnwrappedPrimitive {
        positions: unwrapped.positions,
        normals,
        tangents,
        uvs: unwrapped.uvs,
        indices: unwrapped.indices,

        material: primitive.material.clone(),

        vertex_xrefs: unwrapped.vertex_xrefs,
        atlas_indices: unwrapped.atlas_indices,
        chart_indices: unwrapped.chart_indices,

        atlas_count: unwrapped.atlas_count,
        atlas_width: unwrapped.atlas_width,
        atlas_height: unwrapped.atlas_height,
        chart_count: unwrapped.chart_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwraps_cube() {
        let positions = vec![
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ];

        let indices = vec![
            // Front
            0, 1, 2, 0, 2, 3, // Back
            5, 4, 7, 5, 7, 6, // Left
            4, 0, 3, 4, 3, 7, // Right
            1, 5, 6, 1, 6, 2, // Top
            3, 2, 6, 3, 6, 7, // Bottom
            4, 5, 1, 4, 1, 0,
        ];

        let input = UnwrapInput { positions, indices };

        let result = unwrap_mesh(&input).expect("xatlas should successfully unwrap a cube");

        assert!(!result.positions.is_empty());
        assert!(!result.indices.is_empty());
        assert!(!result.uvs.is_empty());

        assert_eq!(result.positions.len(), result.uvs.len());

        assert_eq!(result.positions.len(), result.vertex_xrefs.len());

        assert_eq!(result.indices.len() % 3, 0);

        for &index in &result.indices {
            assert!(
                (index as usize) < result.positions.len(),
                "Generated index {index} is out of bounds"
            );
        }

        for uv in &result.uvs {
            assert!(uv[0].is_finite());
            assert!(uv[1].is_finite());
        }

        for &xref in &result.vertex_xrefs {
            assert!(
                (xref as usize) < input.positions.len(),
                "Invalid xatlas xref: {xref}"
            );
        }

        assert!(result.atlas_count > 0);
        assert!(result.atlas_width > 0);
        assert!(result.atlas_height > 0);
        assert!(result.chart_count > 0);
    }

    #[test]
    fn unwraps_mesh_primitive() {
        let primitive = MeshPrimitive {
            positions: vec![
                [-1.0, -1.0, -1.0],
                [1.0, -1.0, -1.0],
                [1.0, 1.0, -1.0],
                [-1.0, 1.0, -1.0],
                [-1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0],
                [1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0],
            ],

            normals: vec![[0.0, 0.0, 1.0]; 8],

            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 8],

            uvs: vec![[0.0, 0.0]; 8],

            indices: vec![
                0, 1, 2, 0, 2, 3, 5, 4, 7, 5, 7, 6, 4, 0, 3, 4, 3, 7, 1, 5, 6, 1, 6, 2, 3, 2, 6, 3,
                6, 7, 4, 5, 1, 4, 1, 0,
            ],

            material: Some(MeshMaterialRef {
                id: 7,
                name: "TestMaterial".to_owned(),
            }),
        };

        let result = unwrap_primitive(&primitive).expect("xatlas should unwrap the primitive");

        assert!(!result.positions.is_empty());
        assert!(!result.indices.is_empty());
        assert!(!result.uvs.is_empty());

        assert_eq!(result.positions.len(), result.normals.len());

        assert_eq!(result.positions.len(), result.tangents.len());

        assert_eq!(result.positions.len(), result.uvs.len());

        assert_eq!(result.positions.len(), result.vertex_xrefs.len());

        assert_eq!(result.material.as_ref().unwrap().id, 7);

        for &xref in &result.vertex_xrefs {
            assert!((xref as usize) < primitive.positions.len());
        }

        for &index in &result.indices {
            assert!((index as usize) < result.positions.len());
        }
    }
}

pub fn unwrap_scene(scene: &MeshScene) -> anyhow::Result<UnwrappedMeshScene> {
    let mut objects = Vec::with_capacity(scene.objects.len());

    for object in &scene.objects {
        let mut primitives = Vec::with_capacity(object.primitives.len());

        for primitive in &object.primitives {
            primitives.push(unwrap_primitive(primitive)?);
        }

        objects.push(UnwrappedMeshObject {
            name: object.name.clone(),
            transform: object.transform,
            primitives,
        });
    }

    Ok(UnwrappedMeshScene { objects })
}

pub fn unwrap_current_mesh(
    mesh_preview: &mut MeshPreview,
    mesh_id: u64,
    scene: &super::import::MeshScene,
) -> anyhow::Result<()> {
    let unwrapped = super::unwrap::unwrap_scene(scene)?;

    mesh_preview.unwrapped_scene = Some(unwrapped);
    mesh_preview.unwrapped_mesh_id = Some(mesh_id);
    mesh_preview.uv_mode = MeshUvMode::Unwrapped;

    Ok(())
}
