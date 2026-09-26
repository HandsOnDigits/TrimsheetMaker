use std::path::Path;

use gltf::mesh::util::ReadIndices;

use super::{
    MeshImportError, MeshMaterialRef, MeshObject, MeshPrimitive, MeshScene, MeshTransform,
};

pub fn import(path: &Path) -> Result<MeshScene, MeshImportError> {
    let (document, buffers, _images) =
        gltf::import(path).map_err(|error| MeshImportError::Other(error.to_string()))?;

    let mut objects = Vec::new();

    for scene in document.scenes() {
        for node in scene.nodes() {
            collect_node(node, &buffers, &mut objects)?;
        }
    }

    Ok(MeshScene { objects })
}

fn collect_node(
    node: gltf::Node<'_>,
    buffers: &[gltf::buffer::Data],
    objects: &mut Vec<MeshObject>,
) -> Result<(), MeshImportError> {
    let transform = mesh_transform_from_gltf(node.transform());

    let mut primitives = Vec::new();

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            primitives.push(convert_primitive(&primitive, buffers)?);
        }
    }

    if !primitives.is_empty() {
        let name = node
            .name()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("Node {}", node.index()));

        objects.push(MeshObject {
            name,
            transform,
            primitives,
        });
    }

    for child in node.children() {
        collect_node(child, buffers, objects)?;
    }

    Ok(())
}

fn convert_primitive(
    primitive: &gltf::Primitive<'_>,
    buffers: &[gltf::buffer::Data],
) -> Result<MeshPrimitive, MeshImportError> {
    if primitive.mode() != gltf::mesh::Mode::Triangles {
        return Err(MeshImportError::InvalidData(format!(
            "unsupported glTF primitive mode {:?}; only triangles are supported",
            primitive.mode()
        )));
    }

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));

    let positions = reader
        .read_positions()
        .ok_or_else(|| {
            MeshImportError::InvalidData("glTF primitive has no vertex positions".to_owned())
        })?
        .map(|position| [position[0], position[1], position[2]])
        .collect::<Vec<_>>();

    if positions.is_empty() {
        return Err(MeshImportError::InvalidData(
            "glTF primitive contains no vertices".to_owned(),
        ));
    }

    let normals = match reader.read_normals() {
        Some(normals) => normals
            .map(|normal| [normal[0], normal[1], normal[2]])
            .collect::<Vec<_>>(),
        None => vec![[0.0, 0.0, 0.0]; positions.len()],
    };

    if normals.len() != positions.len() {
        return Err(MeshImportError::InvalidData(
            "glTF primitive normal count does not match vertex count".to_owned(),
        ));
    }

    let tangents = match reader.read_tangents() {
        Some(tangents) => tangents
            .map(|tangent| [tangent[0], tangent[1], tangent[2], tangent[3]])
            .collect::<Vec<_>>(),
        None => vec![[0.0, 0.0, 0.0, 1.0]; positions.len()],
    };

    if tangents.len() != positions.len() {
        return Err(MeshImportError::InvalidData(
            "glTF primitive tangent count does not match vertex count".to_owned(),
        ));
    }

    let uvs = match reader.read_tex_coords(0) {
        Some(gltf::mesh::util::ReadTexCoords::U8(iter)) => iter
            .map(|uv| [uv[0] as f32 / 255.0, uv[1] as f32 / 255.0])
            .collect::<Vec<_>>(),

        Some(gltf::mesh::util::ReadTexCoords::U16(iter)) => iter
            .map(|uv| [uv[0] as f32 / 65535.0, uv[1] as f32 / 65535.0])
            .collect::<Vec<_>>(),

        Some(gltf::mesh::util::ReadTexCoords::F32(iter)) => {
            iter.map(|uv| [uv[0], uv[1]]).collect::<Vec<_>>()
        }

        None => vec![[0.0, 0.0]; positions.len()],
    };

    if uvs.len() != positions.len() {
        return Err(MeshImportError::InvalidData(
            "glTF primitive UV count does not match vertex count".to_owned(),
        ));
    }

    let indices = match reader.read_indices() {
        Some(ReadIndices::U8(iter)) => iter.map(u32::from).collect::<Vec<_>>(),
        Some(ReadIndices::U16(iter)) => iter.map(u32::from).collect::<Vec<_>>(),
        Some(ReadIndices::U32(iter)) => iter.collect::<Vec<_>>(),
        None => (0..positions.len() as u32).collect::<Vec<_>>(),
    };

    if indices.len() % 3 != 0 {
        return Err(MeshImportError::InvalidData(
            "glTF primitive index count is not divisible by 3".to_owned(),
        ));
    }

    for &index in &indices {
        if index as usize >= positions.len() {
            return Err(MeshImportError::InvalidData(format!(
                "glTF primitive references invalid vertex index {}",
                index
            )));
        }
    }

    let material = primitive.material();

    let material_id = material.index().unwrap_or(0);

    let material_name = material
        .name()
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("Material {}", material_id));

    let material = Some(MeshMaterialRef {
        id: material_id,
        name: material_name,
    });

    Ok(MeshPrimitive {
        positions,
        normals,
        tangents,
        uvs,
        indices,
        material,
    })
}

fn mesh_transform_from_gltf(transform: gltf::scene::Transform) -> MeshTransform {
    let matrix = transform.clone().matrix();

    let translation = [matrix[3][0], matrix[3][1], matrix[3][2]];

    let scale = [
        length3([matrix[0][0], matrix[0][1], matrix[0][2]]),
        length3([matrix[1][0], matrix[1][1], matrix[1][2]]),
        length3([matrix[2][0], matrix[2][1], matrix[2][2]]),
    ];

    let rotation = match transform {
        gltf::scene::Transform::Decomposed { rotation, .. } => rotation,

        gltf::scene::Transform::Matrix { .. } => matrix_to_quaternion(matrix, scale),
    };

    MeshTransform {
        translation,
        rotation,
        scale,
    }
}

fn length3(value: [f32; 3]) -> f32 {
    (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt()
}

fn matrix_to_quaternion(matrix: [[f32; 4]; 4], scale: [f32; 3]) -> [f32; 4] {
    let sx = if scale[0] != 0.0 { scale[0] } else { 1.0 };
    let sy = if scale[1] != 0.0 { scale[1] } else { 1.0 };
    let sz = if scale[2] != 0.0 { scale[2] } else { 1.0 };

    let m00 = matrix[0][0] / sx;
    let m01 = matrix[0][1] / sy;
    let m02 = matrix[0][2] / sz;

    let m10 = matrix[1][0] / sx;
    let m11 = matrix[1][1] / sy;
    let m12 = matrix[1][2] / sz;

    let m20 = matrix[2][0] / sx;
    let m21 = matrix[2][1] / sy;
    let m22 = matrix[2][2] / sz;

    let trace = m00 + m11 + m22;

    let (x, y, z, w);

    if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;

        w = 0.25 * s;
        x = (m21 - m12) / s;
        y = (m02 - m20) / s;
        z = (m10 - m01) / s;
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;

        w = (m21 - m12) / s;
        x = 0.25 * s;
        y = (m01 + m10) / s;
        z = (m02 + m20) / s;
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;

        w = (m02 - m20) / s;
        x = (m01 + m10) / s;
        y = 0.25 * s;
        z = (m12 + m21) / s;
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;

        w = (m10 - m01) / s;
        x = (m02 + m20) / s;
        y = (m12 + m21) / s;
        z = 0.25 * s;
    }

    [x, y, z, w]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn import_gltf_scene() {
        let mesh_path =
            env::var("GLTF_TEST_MODEL").expect("GLTF_TEST_MODEL environment variable is not set");

        let scene = import(Path::new(&mesh_path)).expect("failed to import glTF scene");

        println!("Imported {} mesh objects", scene.objects.len());

        assert!(
            !scene.objects.is_empty(),
            "glTF scene contains no mesh objects"
        );

        for object in &scene.objects {
            println!(
                "Object '{}' - {} primitives",
                object.name,
                object.primitives.len()
            );

            assert!(
                !object.primitives.is_empty(),
                "object '{}' contains no primitives",
                object.name
            );

            for (index, primitive) in object.primitives.iter().enumerate() {
                if let Some(material) = &primitive.material {
                    println!("    Material: id={}, name='{}'", material.id, material.name);

                    assert!(!material.name.is_empty());
                }

                println!(
                    "  Primitive {}: {} vertices, {} indices, material {:?}",
                    index,
                    primitive.positions.len(),
                    primitive.indices.len(),
                    primitive.material
                );

                assert!(!primitive.positions.is_empty());

                assert_eq!(primitive.positions.len(), primitive.normals.len());

                assert_eq!(primitive.positions.len(), primitive.tangents.len());

                assert_eq!(primitive.positions.len(), primitive.uvs.len());

                assert_eq!(
                    primitive.indices.len() % 3,
                    0,
                    "primitive {} has a non-triangle index count",
                    index
                );

                for &index in &primitive.indices {
                    assert!(
                        (index as usize) < primitive.positions.len(),
                        "primitive references vertex {} but only {} vertices exist",
                        index,
                        primitive.positions.len()
                    );
                }
            }
        }
    }

    #[test]
    fn convert_gltf_transforms() {
        let transform = gltf::scene::Transform::Decomposed {
            translation: [10.0, 20.0, 30.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [2.0, 3.0, 4.0],
        };

        let result = mesh_transform_from_gltf(transform);

        assert_eq!(result.translation, [10.0, 20.0, 30.0]);
        assert_eq!(result.rotation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(result.scale, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn convert_gltf_matrix_transform() {
        let matrix = [
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 3.0, 0.0, 0.0],
            [0.0, 0.0, 4.0, 0.0],
            [10.0, 20.0, 30.0, 1.0],
        ];

        let transform = gltf::scene::Transform::Matrix { matrix };

        let result = mesh_transform_from_gltf(transform);

        assert_eq!(result.translation, [10.0, 20.0, 30.0]);

        assert_eq!(result.scale, [2.0, 3.0, 4.0]);

        assert_eq!(result.rotation, [0.0, 0.0, 0.0, 1.0]);
    }
}
