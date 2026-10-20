use bevy::prelude::*;

use anyhow::{Context, ensure};
use serde_json::{Map, Value, json};
use std::{collections::HashMap, fs::File, io::Write, path::Path};

use crate::{
    data::Atlas,
    mesh::{MeshEntry, MeshSceneState, import::MeshPrimitive, uv::transform_uvs},
};

const ARRAY_BUFFER: u32 = 34962;
const ELEMENT_ARRAY_BUFFER: u32 = 34963;
const FLOAT: u32 = 5126;
const UNSIGNED_INT: u32 = 5125;

/// Export the visible mesh scene to a geometry-only GLB.
///
/// Each original MeshObject becomes one combined mesh.
/// Its primitives are merged and compatible vertices are welded,
/// but vertices are never welded across different MeshObjects.
///
/// No materials or textures are written to the GLB.
pub fn export_glb(
    path: impl AsRef<Path>,
    scene_state: &MeshSceneState,
    atlas: &Atlas,
) -> anyhow::Result<()> {
    let mut binary = GlbBinary::default();

    let mut nodes: Vec<Value> = Vec::new();
    let mut meshes: Vec<Value> = Vec::new();
    let mut scene_roots: Vec<usize> = Vec::new();

    for entry in &scene_state.meshes {
        // Follow the editor's mesh visibility state.
        if !entry.visible {
            continue;
        }

        let mut object_nodes = Vec::new();

        for (object_index, object) in entry.scene.objects.iter().enumerate() {
            // Respect per-primitive visibility when runtime part metadata
            // is available. Restored scenes with no part metadata export
            // their stored primitives.
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

            // Deliberately omit "material". This primitive has no
            // exported material or texture dependencies.
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
            // Preserve the grouping of each imported MeshEntry.
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

struct ProcessedPrimitive<'a> {
    source: &'a MeshPrimitive,
    transformed_uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

struct CombinedMesh {
    positions: Vec<[f32; 3]>,
    normals: Option<Vec<[f32; 3]>>,
    tangents: Option<Vec<[f32; 4]>>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

/// Combine all selected primitives belonging to one original object.
///
/// This function is called separately for each object. Its weld map is
/// therefore never shared between different objects.
fn combine_object(
    entry: &MeshEntry,
    object_index: usize,
    primitives: &[(usize, &MeshPrimitive)],
    atlas: &Atlas,
) -> anyhow::Result<Option<CombinedMesh>> {
    let object = &entry.scene.objects[object_index];

    let mut processed = Vec::new();

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
            primitive.positions.iter().all(finite),
            "Object '{}' primitive {} contains non-finite positions",
            object.name,
            primitive_index
        );

        let indices = if primitive.indices.is_empty() {
            ensure!(
                primitive.positions.len() % 3 == 0,
                "Object '{}' primitive {} has no indices and its vertex count is not divisible by 3",
                object.name,
                primitive_index
            );

            (0..primitive.positions.len())
                .map(|index| u32::try_from(index).context("Primitive has too many vertices"))
                .collect::<anyhow::Result<Vec<_>>>()?
        } else {
            primitive.indices.clone()
        };

        ensure!(
            indices.len() % 3 == 0,
            "Object '{}' primitive {} does not contain triangle-list indices",
            object.name,
            primitive_index
        );

        for &index in &indices {
            ensure!(
                (index as usize) < primitive.positions.len(),
                "Object '{}' primitive {} references invalid vertex {}",
                object.name,
                primitive_index,
                index
            );
        }

        // Find the exact assignment used by the viewport.
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

        // Apply UV transforms before merging vertices. Different primitives
        // may use different atlas regions or independent UV transformations.
        let transformed_uvs = match assignment {
            Some((material, assignment)) => {
                let mut uvs = transform_uvs(
                    &primitive.uvs,
                    &assignment.uv_transform,
                    &material.transform,
                    atlas.width,
                    atlas.height,
                );

                // Correct the exported V scale to use the atlas width.
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

        ensure!(
            transformed_uvs.len() == primitive.positions.len(),
            "UV transformation produced an invalid vertex count"
        );

        ensure!(
            transformed_uvs.iter().all(finite),
            "Object '{}' primitive {} contains non-finite UVs",
            object.name,
            primitive_index
        );

        processed.push(ProcessedPrimitive {
            source: primitive,
            transformed_uvs,
            indices,
        });
    }

    if processed.is_empty() {
        return Ok(None);
    }

    // glTF attributes are specified for every vertex in a primitive.
    // If an object has incomplete normals or tangents, omit that attribute
    // from the combined mesh rather than writing a partial attribute array.
    let has_normals = processed
        .iter()
        .all(|primitive| primitive.source.normals.len() == primitive.source.positions.len());

    let has_tangents = has_normals
        && processed
            .iter()
            .all(|primitive| primitive.source.tangents.len() == primitive.source.positions.len());

    if has_normals {
        ensure!(
            processed
                .iter()
                .all(|primitive| { primitive.source.normals.iter().all(finite) }),
            "Object '{}' contains non-finite normals",
            object.name
        );
    }

    if has_tangents {
        ensure!(
            processed
                .iter()
                .all(|primitive| { primitive.source.tangents.iter().all(finite) }),
            "Object '{}' contains non-finite tangents",
            object.name
        );
    }

    let mut combined = CombinedMesh {
        positions: Vec::new(),
        normals: has_normals.then(Vec::new),
        tangents: has_tangents.then(Vec::new),
        uvs: Vec::new(),
        indices: Vec::new(),
    };

    let mut weld_map: HashMap<VertexKey, u32> = HashMap::new();

    for primitive in &processed {
        for &source_index in &primitive.indices {
            let source_index = source_index as usize;

            let position = primitive.source.positions[source_index];
            let uv = primitive.transformed_uvs[source_index];

            let normal = if has_normals {
                Some(primitive.source.normals[source_index])
            } else {
                None
            };

            let tangent = if has_tangents {
                Some(primitive.source.tangents[source_index])
            } else {
                None
            };

            let key = VertexKey {
                position: float_bits(position),
                normal: normal.map(float_bits),
                tangent: tangent.map(float_bits),
                uv: float_bits(uv),
            };

            let output_index = if let Some(&existing) = weld_map.get(&key) {
                existing
            } else {
                let new_index = u32::try_from(combined.positions.len())
                    .context("Combined object has too many vertices for GLB")?;

                combined.positions.push(position);
                combined.uvs.push(uv);

                if let (Some(output), Some(value)) = (&mut combined.normals, normal) {
                    output.push(value);
                }

                if let (Some(output), Some(value)) = (&mut combined.tangents, tangent) {
                    output.push(value);
                }

                weld_map.insert(key, new_index);
                new_index
            };

            combined.indices.push(output_index);
        }
    }

    ensure!(
        !combined.positions.is_empty() && !combined.indices.is_empty(),
        "Object '{}' contains no exportable triangles",
        object.name
    );

    Ok(Some(combined))
}

#[derive(Hash, PartialEq, Eq)]
struct VertexKey {
    position: [u32; 3],
    normal: Option<[u32; 3]>,
    tangent: Option<[u32; 4]>,
    uv: [u32; 2],
}

/// Convert float attributes to hashable bit representations.
///
/// Negative zero is canonicalized to positive zero. Other values must match
/// exactly, so nearby but distinct vertices are not accidentally merged.
fn float_bits<const N: usize>(values: [f32; N]) -> [u32; N] {
    values.map(|value| if value == 0.0 { 0 } else { value.to_bits() })
}

fn finite<const N: usize>(values: &[f32; N]) -> bool {
    values.iter().all(|value| value.is_finite())
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
