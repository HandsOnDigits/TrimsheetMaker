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
const POSITION_WELD_TOLERANCE: f32 = 1.0e-3;

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

/// Combine selected primitives belonging to one original object.
///
/// First weld positions within POSITION_WELD_TOLERANCE. Then expand the
/// indexed triangles into corner streams and use meshopt to deduplicate
/// render vertices using all available attributes.
fn combine_object(
    entry: &MeshEntry,
    object_index: usize,
    primitives: &[(usize, &MeshPrimitive)],
    atlas: &Atlas,
) -> anyhow::Result<Option<CombinedMesh>> {
    let object = &entry.scene.objects[object_index];

    let mut positions = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    let mut indices = Vec::<u32>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut tangents = Vec::<[f32; 4]>::new();

    let mut has_normals = true;
    let mut has_tangents = true;

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

        ensure!(
            primitive.positions.iter().flatten().all(|v| v.is_finite()),
            "Object '{}' primitive {} contains non-finite positions",
            object.name,
            primitive_index
        );

        let base = u32::try_from(positions.len())
            .context("Too many vertices before combining primitives")?;

        let source_indices = if primitive.indices.is_empty() {
            (0..primitive.positions.len())
                .map(|i| u32::try_from(i).context("Primitive has too many vertices"))
                .collect::<anyhow::Result<Vec<_>>>()?
        } else {
            primitive.indices.clone()
        };

        ensure!(
            source_indices.len() % 3 == 0,
            "Object '{}' primitive {} has a non-triangle index count",
            object.name,
            primitive_index
        );

        for &index in &source_indices {
            ensure!(
                (index as usize) < primitive.positions.len(),
                "Object '{}' primitive {} has an out-of-range index",
                object.name,
                primitive_index
            );
        }

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
                let mut result = transform_uvs(
                    &primitive.uvs,
                    &assignment.uv_transform,
                    &material.transform,
                    atlas.width,
                    atlas.height,
                );

                if atlas.height > 0 {
                    let scale = atlas.height as f32 / atlas.width as f32;

                    for uv in &mut result {
                        uv[1] *= scale;
                    }
                }

                result
            }
            None => primitive.uvs.clone(),
        };

        positions.extend_from_slice(&primitive.positions);
        uvs.extend_from_slice(&transformed_uvs);

        if primitive.normals.len() == primitive.positions.len() {
            normals.extend_from_slice(&primitive.normals);
        } else {
            has_normals = false;
        }

        if primitive.tangents.len() == primitive.positions.len() {
            tangents.extend_from_slice(&primitive.tangents);
        } else {
            has_tangents = false;
        }

        for index in source_indices {
            indices.push(base + index);
        }
    }

    if positions.is_empty() || indices.is_empty() {
        return Ok(None);
    }

    if !has_normals {
        normals.clear();
    }

    if !has_tangents || !has_normals {
        tangents.clear();
        has_tangents = false;
    }

    // Meshopt identifies exact position matches. The spatial search below
    // additionally welds positions that are within the configured tolerance.
    let position_bytes = meshopt::utilities::typed_to_bytes(&positions);

    let position_adapter =
        meshopt::VertexDataAdapter::new(position_bytes, std::mem::size_of::<[f32; 3]>(), 0)
            .context("Failed to create meshopt position adapter")?;

    let exact_remap = meshopt::remap::generate_position_remap(&position_adapter);

    let tolerance = POSITION_WELD_TOLERANCE;
    let tolerance_squared = tolerance * tolerance;

    let mut geometric_positions = Vec::<[f32; 3]>::new();
    let mut exact_to_geometric = std::collections::HashMap::<u32, u32>::new();
    let mut spatial_map = std::collections::HashMap::<[i64; 3], Vec<u32>>::new();
    let mut source_to_geometric = vec![0u32; positions.len()];

    for source_index in 0..positions.len() {
        let position = positions[source_index];
        let exact_id = exact_remap[source_index];

        if let Some(&existing) = exact_to_geometric.get(&exact_id) {
            source_to_geometric[source_index] = existing;
            continue;
        }

        let cell = spatial_cell(position, tolerance);
        let mut found = None;

        'search: for dx in -1_i64..=1 {
            for dy in -1_i64..=1 {
                for dz in -1_i64..=1 {
                    let neighbor = [cell[0] + dx, cell[1] + dy, cell[2] + dz];

                    if let Some(candidates) = spatial_map.get(&neighbor) {
                        for &candidate in candidates {
                            let other = geometric_positions[candidate as usize];

                            let dx = position[0] - other[0];
                            let dy = position[1] - other[1];
                            let dz = position[2] - other[2];

                            if dx * dx + dy * dy + dz * dz <= tolerance_squared {
                                found = Some(candidate);
                                break 'search;
                            }
                        }
                    }
                }
            }
        }

        let geometric_index = match found {
            Some(index) => index,
            None => {
                let index = u32::try_from(geometric_positions.len())
                    .context("Too many geometric vertices")?;

                geometric_positions.push(position);
                spatial_map.entry(cell).or_default().push(index);
                index
            }
        };

        exact_to_geometric.insert(exact_id, geometric_index);
        source_to_geometric[source_index] = geometric_index;
    }

    // Render welding preserves UV seams.
    //
    // Vertices can share a geometric position, but must remain separate
    // render vertices when their UVs, normals, or tangents differ.
    let mut output_positions = Vec::<[f32; 3]>::new();
    let mut output_uvs = Vec::<[f32; 2]>::new();
    let mut output_normals = has_normals.then(Vec::<[f32; 3]>::new);
    let mut output_tangents = has_tangents.then(Vec::<[f32; 4]>::new);

    let mut render_vertex_map = std::collections::HashMap::new();
    let mut source_to_render = vec![0u32; positions.len()];

    for source_index in 0..positions.len() {
        let geometric_index = source_to_geometric[source_index];

        // Use the exact attribute bits as part of the key. This allows
        // coincident positions to have different UVs at texture seams.
        let key = (
            geometric_index,
            uvs[source_index].map(f32::to_bits),
            has_normals.then(|| normals[source_index].map(f32::to_bits)),
            has_tangents.then(|| tangents[source_index].map(f32::to_bits)),
        );

        let render_index = if let Some(&existing) = render_vertex_map.get(&key) {
            existing
        } else {
            let index =
                u32::try_from(output_positions.len()).context("Too many render vertices")?;

            output_positions.push(geometric_positions[geometric_index as usize]);
            output_uvs.push(uvs[source_index]);

            if let Some(output) = &mut output_normals {
                output.push(normals[source_index]);
            }

            if let Some(output) = &mut output_tangents {
                output.push(tangents[source_index]);
            }

            render_vertex_map.insert(key, index);
            index
        };

        source_to_render[source_index] = render_index;
    }

    let output_indices: Vec<u32> = indices
        .iter()
        .map(|&source_index| source_to_render[source_index as usize])
        .collect();

    info!(
        "GLB export '{}', object '{}': {} source vertices, {} welded positions, {} render vertices",
        entry.name,
        object.name,
        positions.len(),
        geometric_positions.len(),
        output_positions.len()
    );

    Ok(Some(CombinedMesh {
        positions: output_positions,
        normals: output_normals,
        tangents: output_tangents,
        uvs: output_uvs,
        indices: output_indices,
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
