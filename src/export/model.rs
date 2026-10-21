use bevy::prelude::*;

use anyhow::{Context, ensure};
use serde_json::{Map, Value, json};
use std::{fs::File, io::Write, path::Path};

use crate::{
    data::Atlas,
    mesh::{MeshEntry, MeshSceneState, import::MeshPrimitive, uv::transform_uvs},
};

struct CombinedMesh {
    positions: Vec<[f32; 3]>,
    normals: Option<Vec<[f32; 3]>>,
    tangents: Option<Vec<[f32; 4]>>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

#[derive(Hash, PartialEq, Eq)]
struct VertexKey {
    geometric_index: u32,
    uv: [i32; 2],
}

#[derive(Default)]
struct GlbBinary {
    bytes: Vec<u8>,
    buffer_views: Vec<Value>,
    accessors: Vec<Value>,
}

impl GlbBinary {
    fn align(&mut self) {
        while self.bytes.len() % 4 != 0 {
            self.bytes.push(0);
        }
    }

    fn add_f32_accessor<const N: usize>(
        &mut self,
        values: &[[f32; N]],
        accessor_type: &str,
        target: u32,
        include_bounds: bool,
    ) -> usize {
        self.align();

        let byte_offset = self.bytes.len();

        for value in values {
            for component in value {
                self.bytes.extend_from_slice(&component.to_le_bytes());
            }
        }

        let byte_length = self.bytes.len() - byte_offset;

        let view_index = self.buffer_views.len();

        self.buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": byte_offset,
            "byteLength": byte_length,
            "target": target
        }));

        let mut accessor = json!({
            "bufferView": view_index,
            "byteOffset": 0,
            "componentType": FLOAT,
            "count": values.len(),
            "type": accessor_type
        });

        if include_bounds && !values.is_empty() {
            let mut min = [f32::INFINITY; N];
            let mut max = [f32::NEG_INFINITY; N];

            for value in values {
                for component in 0..N {
                    min[component] = min[component].min(value[component]);
                    max[component] = max[component].max(value[component]);
                }
            }

            accessor["min"] = json!(min.as_slice());
            accessor["max"] = json!(max.as_slice());
        }

        let accessor_index = self.accessors.len();
        self.accessors.push(accessor);

        accessor_index
    }

    fn add_index_accessor(&mut self, indices: &[u32]) -> usize {
        self.align();

        let byte_offset = self.bytes.len();

        for index in indices {
            self.bytes.extend_from_slice(&index.to_le_bytes());
        }

        let byte_length = self.bytes.len() - byte_offset;

        let view_index = self.buffer_views.len();

        self.buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": byte_offset,
            "byteLength": byte_length,
            "target": ELEMENT_ARRAY_BUFFER
        }));

        let accessor_index = self.accessors.len();

        self.accessors.push(json!({
            "bufferView": view_index,
            "byteOffset": 0,
            "componentType": UNSIGNED_INT,
            "count": indices.len(),
            "type": "SCALAR"
        }));

        accessor_index
    }
}

const ARRAY_BUFFER: u32 = 34962;
const ELEMENT_ARRAY_BUFFER: u32 = 34963;
const FLOAT: u32 = 5126;
const UNSIGNED_INT: u32 = 5125;
const POSITION_WELD_TOLERANCE: f32 = 5.0e-3;
const UV_WELD_TOLERANCE: f32 = 1.0e-3;

fn quantize(value: f32, tolerance: f32) -> i32 {
    (value / tolerance).round() as i32
}

fn quantize_uv(uv: [f32; 2]) -> [i32; 2] {
    [
        quantize(uv[0], UV_WELD_TOLERANCE),
        quantize(uv[1], UV_WELD_TOLERANCE),
    ]
}

fn spatial_cell(position: [f32; 3], cell_size: f32) -> [i64; 3] {
    position.map(|value| ((value as f64 / cell_size as f64).floor()) as i64)
}

/// Export the visible mesh scene to a geometry-only GLB.
fn export_glb(
    path: impl AsRef<Path>,
    scene_state: &MeshSceneState,
    atlas: &Atlas,
) -> anyhow::Result<()> {
    let mut binary = GlbBinary::default();

    let mut nodes: Vec<Value> = Vec::new();
    let mut meshes: Vec<Value> = Vec::new();
    let mut scene_roots: Vec<usize> = Vec::new();

    for entry in &scene_state.meshes {
        if !entry.visible {
            continue;
        }

        let mut object_nodes = Vec::new();

        for (object_index, object) in entry.scene.objects.iter().enumerate() {
            let selected_primitives: Vec<(usize, &MeshPrimitive)> = object
                .primitives
                .iter()
                .enumerate()
                .filter(|(primitive_index, _)| {
                    primitive_is_visible(entry, object_index, *primitive_index)
                })
                .map(|(index, primitive)| (index, primitive))
                .collect();

            if selected_primitives.is_empty() {
                continue;
            }

            let Some(combined) = combine_object(entry, object_index, &selected_primitives, atlas)?
            else {
                continue;
            };

            validate_object_transform(entry, object_index)?;

            let position_accessor =
                binary.add_f32_accessor(&combined.positions, "VEC3", ARRAY_BUFFER, true);

            let mut attributes = Map::new();
            attributes.insert("POSITION".to_owned(), json!(position_accessor));

            if let Some(normals) = &combined.normals {
                let accessor = binary.add_f32_accessor(normals, "VEC3", ARRAY_BUFFER, false);
                attributes.insert("NORMAL".to_owned(), json!(accessor));
            }

            if let Some(tangents) = &combined.tangents {
                let accessor = binary.add_f32_accessor(tangents, "VEC4", ARRAY_BUFFER, false);
                attributes.insert("TANGENT".to_owned(), json!(accessor));
            }

            let uv_accessor = binary.add_f32_accessor(&combined.uvs, "VEC2", ARRAY_BUFFER, false);
            attributes.insert("TEXCOORD_0".to_owned(), json!(uv_accessor));

            let indices_accessor = binary.add_index_accessor(&combined.indices);
            let mesh_index = meshes.len();

            meshes.push(json!({
                "name": object.name,
                "primitives": [{
                    "attributes": attributes,
                    "indices": indices_accessor,
                    "mode": 4
                }]
            }));

            let transform = object.transform;

            nodes.push(json!({
                "name": object.name,
                "mesh": mesh_index,
                "translation": transform.translation,
                "rotation": transform.rotation,
                "scale": transform.scale
            }));

            object_nodes.push(nodes.len() - 1);
        }

        if !object_nodes.is_empty() {
            nodes.push(json!({
                "name": entry.name,
                "children": object_nodes
            }));

            scene_roots.push(nodes.len() - 1);
        }
    }

    ensure!(
        !scene_roots.is_empty(),
        "There are no visible mesh objects with exportable geometry."
    );

    let document = json!({
        "asset": {
            "version": "2.0",
            "generator": "TrimsheetMaker"
        },
        "scene": 0,
        "scenes": [{
            "nodes": scene_roots
        }],
        "nodes": nodes,
        "meshes": meshes,
        "accessors": binary.accessors,
        "bufferViews": binary.buffer_views,
        "buffers": [{
            "byteLength": binary.bytes.len()
        }]
    });

    write_glb(path.as_ref(), &document, &binary.bytes).context("Failed to write GLB model export")
}

fn primitive_is_visible(entry: &MeshEntry, object_index: usize, primitive_index: usize) -> bool {
    if entry.parts.is_empty() {
        return true;
    }

    entry
        .parts
        .iter()
        .find(|part| part.object_index == object_index && part.primitive_index == primitive_index)
        .map(|part| part.visible)
        .unwrap_or(true)
}

fn validate_object_transform(entry: &MeshEntry, object_index: usize) -> anyhow::Result<()> {
    let object = &entry.scene.objects[object_index];
    let transform = object.transform;

    ensure!(
        transform.translation.iter().all(|v| v.is_finite())
            && transform.rotation.iter().all(|v| v.is_finite())
            && transform.scale.iter().all(|v| v.is_finite()),
        "Object '{}' in '{}' has a non-finite transform",
        object.name,
        entry.name
    );

    Ok(())
}

/// Combine all selected primitives belonging to one original object,
/// using a robust union-find vertex welding approach to eliminate gaps and ensure a watertight mesh.
fn combine_object(
    entry: &MeshEntry,
    object_index: usize,
    primitives: &[(usize, &MeshPrimitive)],
    atlas: &Atlas,
) -> anyhow::Result<Option<CombinedMesh>> {
    let object = &entry.scene.objects[object_index];

    let mut all_positions = Vec::new();
    let mut all_uvs = Vec::new();
    let mut all_indices = Vec::new();

    for &(primitive_index, primitive) in primitives {
        if primitive.positions.is_empty() {
            continue;
        }

        ensure!(
            primitive.uvs.len() == primitive.positions.len(),
            "Object '{}' primitive {} has {} UVs for {} vertices",
            object.name,
            primitive_index,
            primitive.uvs.len(),
            primitive.positions.len()
        );

        let base_index = u32::try_from(all_positions.len())
            .context("Too many vertices before combining primitives")?;

        let indices = if primitive.indices.is_empty() {
            (0..primitive.positions.len())
                .map(|i| u32::try_from(i).context("Primitive has too many vertices"))
                .collect::<anyhow::Result<Vec<_>>>()?
        } else {
            primitive.indices.clone()
        };

        let assignment = atlas.materials.iter().find_map(|material| {
            material
                .mesh_assignments
                .iter()
                .find(|assignment| {
                    assignment.mesh_id == entry.id
                        && assignment.object_index == object_index
                        && assignment.primitive_index == primitive_index
                })
                .map(|assignment| (material, assignment))
        });

        let transformed_uvs = match assignment {
            Some((material, assignment)) => {
                let mut uvs = transform_uvs(
                    &primitive.uvs,
                    &assignment.uv_transform,
                    &material.transform,
                    atlas.width,
                    atlas.height,
                );

                if atlas.height > 0 {
                    let scale = atlas.height as f32 / atlas.width as f32;
                    for uv in &mut uvs {
                        uv[1] *= scale;
                    }
                }
                uvs
            }
            None => primitive.uvs.clone(),
        };

        all_positions.extend_from_slice(&primitive.positions);
        all_uvs.extend_from_slice(&transformed_uvs);

        for &index in &indices {
            all_indices.push(base_index + index);
        }
    }

    if all_positions.is_empty() || all_indices.is_empty() {
        return Ok(None);
    }

    // --- Robust DSU / Spatial Grid Welding ---
    let vertex_count = all_positions.len();
    let mut parent: Vec<usize> = (0..vertex_count).collect();

    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }

    fn union(parent: &mut [usize], i: usize, j: usize) {
        let root_i = find(parent, i);
        let root_j = find(parent, j);
        if root_i != root_j {
            parent[root_i] = root_j;
        }
    }

    // Spatial binning for efficient neighbor lookup without grid-aliasing gaps
    let mut spatial_map: std::collections::HashMap<[i64; 3], Vec<usize>> =
        std::collections::HashMap::new();
    let tolerance_squared = POSITION_WELD_TOLERANCE * POSITION_WELD_TOLERANCE;

    for (i, &pos) in all_positions.iter().enumerate() {
        let cell = spatial_cell(pos, POSITION_WELD_TOLERANCE);

        for dx in -1_i64..=1 {
            for dy in -1_i64..=1 {
                for dz in -1_i64..=1 {
                    let neighbor_cell = [cell[0] + dx, cell[1] + dy, cell[2] + dz];
                    if let Some(candidates) = spatial_map.get(&neighbor_cell) {
                        for &candidate in candidates {
                            let existing_pos = all_positions[candidate];
                            let dist_sq = (pos[0] - existing_pos[0]).powi(2)
                                + (pos[1] - existing_pos[1]).powi(2)
                                + (pos[2] - existing_pos[2]).powi(2);

                            if dist_sq <= tolerance_squared {
                                union(&mut parent, i, candidate);
                            }
                        }
                    }
                }
            }
        }

        spatial_map.entry(cell).or_default().push(i);
    }

    // Compress roots and map old vertices to new welded geometric indices
    let mut root_to_geometric: std::collections::HashMap<usize, u32> =
        std::collections::HashMap::new();
    let mut geometric_positions: Vec<[f32; 3]> = Vec::new();
    let mut vertex_geometric_indices = vec![0u32; vertex_count];

    for i in 0..vertex_count {
        let root = find(&mut parent, i);
        let geometric_index = *root_to_geometric.entry(root).or_insert_with(|| {
            let idx = u32::try_from(geometric_positions.len()).unwrap();
            geometric_positions.push(all_positions[i]);
            idx
        });
        vertex_geometric_indices[i] = geometric_index;
    }

    // Remove degenerate triangles and duplicate geometric triangles.
    // Keep original corner indices so UVs, normals, and tangents remain attached.
    let mut cleaned_indices = Vec::with_capacity(all_indices.len());
    let mut seen_triangles = std::collections::HashSet::<[u32; 3]>::new();

    for triangle in all_indices.chunks_exact(3) {
        let source = [triangle[0], triangle[1], triangle[2]];

        let geometric = source.map(|index| vertex_geometric_indices[index as usize]);

        // Welding collapsed this triangle to a line or a point.
        if geometric[0] == geometric[1]
            || geometric[1] == geometric[2]
            || geometric[2] == geometric[0]
        {
            continue;
        }

        // Sort only the key, not the actual triangle, to preserve winding.
        let mut canonical = geometric;
        canonical.sort_unstable();

        // Remove duplicate triangles, regardless of their winding.
        if !seen_triangles.insert(canonical) {
            continue;
        }

        cleaned_indices.extend_from_slice(&source);
    }

    // Build render vertices from welded geometry while preserving UV seams.
    // Normals and tangents are omitted so Blender can calculate them after
    // merging coincident vertices.
    let mut render_map: std::collections::HashMap<VertexKey, u32> =
        std::collections::HashMap::new();

    let mut welded_positions = Vec::new();
    let mut welded_uvs = Vec::new();
    let mut welded_indices = Vec::with_capacity(cleaned_indices.len());

    for &old_idx in &cleaned_indices {
        let old_idx = old_idx as usize;
        let geometric_index = vertex_geometric_indices[old_idx];
        let uv = all_uvs[old_idx];

        let key = VertexKey {
            geometric_index,
            uv: quantize_uv(uv),
        };

        let output_index = match render_map.get(&key) {
            Some(&existing) => existing,
            None => {
                let index =
                    u32::try_from(welded_positions.len()).context("Too many render vertices")?;

                welded_positions.push(geometric_positions[geometric_index as usize]);
                welded_uvs.push(uv);

                render_map.insert(key, index);
                index
            }
        };

        welded_indices.push(output_index);
    }

    Ok(Some(CombinedMesh {
        positions: welded_positions,
        normals: None,
        tangents: None,
        uvs: welded_uvs,
        indices: welded_indices,
    }))
}

fn write_glb(path: &Path, document: &Value, binary: &[u8]) -> anyhow::Result<()> {
    let mut json_chunk = serde_json::to_vec(document).context("Failed to encode glTF JSON")?;

    while json_chunk.len() % 4 != 0 {
        json_chunk.push(b' ');
    }

    let mut binary_chunk = binary.to_vec();

    while binary_chunk.len() % 4 != 0 {
        binary_chunk.push(0);
    }

    let total_length = 12usize
        .checked_add(8)
        .and_then(|length| length.checked_add(json_chunk.len()))
        .and_then(|length| length.checked_add(8))
        .and_then(|length| length.checked_add(binary_chunk.len()))
        .context("GLB file size overflow")?;

    let total_length =
        u32::try_from(total_length).context("GLB file exceeds the maximum supported size")?;

    let json_length = u32::try_from(json_chunk.len()).context("GLB JSON chunk is too large")?;

    let binary_length =
        u32::try_from(binary_chunk.len()).context("GLB binary chunk is too large")?;

    let mut file =
        File::create(path).with_context(|| format!("Cannot create '{}'", path.display()))?;

    // GLB header.
    file.write_all(&0x4654_6C67u32.to_le_bytes())?; // "glTF"
    file.write_all(&2u32.to_le_bytes())?;
    file.write_all(&total_length.to_le_bytes())?;

    // JSON chunk.
    file.write_all(&json_length.to_le_bytes())?;
    file.write_all(&0x4E4F_534Au32.to_le_bytes())?; // JSON
    file.write_all(&json_chunk)?;

    // BIN chunk.
    file.write_all(&binary_length.to_le_bytes())?;
    file.write_all(&0x004E_4942u32.to_le_bytes())?; // BIN
    file.write_all(&binary_chunk)?;

    file.flush()?;

    Ok(())
}

pub fn start_model_export(
    task: &mut super::task::ExportTaskState,
    path: std::path::PathBuf,
    scene_state: &crate::mesh::MeshSceneState,
    atlas: &Atlas,
) -> bool {
    if task.is_active() {
        return false;
    }

    let scene = scene_state.clone();
    let atlas = atlas.clone();

    task.start("Exporting model…", move |_| {
        crate::export::model::export_glb(&path, &scene, &atlas)
            .map(|()| format!("Model exported: {}", path.display()))
    })
}
