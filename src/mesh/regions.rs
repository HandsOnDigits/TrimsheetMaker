#[derive(Debug, Clone)]
pub struct MeshRegion {
    pub id: usize,

    /// Index into the imported MeshScene geometry.
    pub object_index: usize,

    /// Index of the primitive within that object.
    pub primitive_index: usize,

    /// AtlasMaterial this region uses.
    pub material_index: usize,
}

#[derive(Clone, Debug, Default)]
pub struct MeshAsset {
    pub regions: Vec<MeshRegion>,
}

pub struct MeshMaterialAssignment {
    pub mesh_region_id: usize,
    pub material_index: usize,
}
