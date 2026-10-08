use std::collections::HashMap;

use super::{MeshScene, import::MeshPrimitive};

use crate::transform::UvTransform;

fn connected_face_components(indices: &[u32]) -> Vec<Vec<usize>> {
    let face_count = indices.len() / 3;

    if face_count == 0 {
        return Vec::new();
    }

    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();

    for face in 0..face_count {
        let base = face * 3;

        let a = indices[base];
        let b = indices[base + 1];
        let c = indices[base + 2];

        for edge in [(a, b), (b, c), (c, a)] {
            let key = if edge.0 <= edge.1 {
                edge
            } else {
                (edge.1, edge.0)
            };

            edge_faces.entry(key).or_default().push(face);
        }
    }

    let mut adjacency = vec![Vec::<usize>::new(); face_count];

    for faces in edge_faces.values() {
        for i in 0..faces.len() {
            for j in (i + 1)..faces.len() {
                adjacency[faces[i]].push(faces[j]);
                adjacency[faces[j]].push(faces[i]);
            }
        }
    }

    let mut visited = vec![false; face_count];
    let mut components = Vec::new();

    for start in 0..face_count {
        if visited[start] {
            continue;
        }

        let mut stack = vec![start];
        let mut component = Vec::new();

        visited[start] = true;

        while let Some(face) = stack.pop() {
            component.push(face);

            for &neighbor in &adjacency[face] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    stack.push(neighbor);
                }
            }
        }

        component.sort_unstable();
        components.push(component);
    }

    components
}

impl MeshPrimitive {
    pub fn split_by_connected_faces(&self) -> Vec<Self> {
        let components = connected_face_components(&self.indices);

        let has_normals = self.normals.len() == self.positions.len();
        let has_tangents = self.tangents.len() == self.positions.len();
        let has_uvs = self.uvs.len() == self.positions.len();

        components
            .into_iter()
            .map(|faces| {
                let mut vertex_remap = HashMap::<u32, u32>::new();

                let mut positions = Vec::new();
                let mut normals = Vec::new();
                let mut tangents = Vec::new();
                let mut uvs = Vec::new();
                let mut indices = Vec::with_capacity(faces.len() * 3);

                for face in faces {
                    let base = face * 3;

                    for &old_index in &self.indices[base..base + 3] {
                        let new_index = if let Some(&index) = vertex_remap.get(&old_index) {
                            index
                        } else {
                            let index = positions.len() as u32;
                            let old = old_index as usize;

                            vertex_remap.insert(old_index, index);

                            positions.push(self.positions[old]);

                            if has_normals {
                                normals.push(self.normals[old]);
                            }

                            if has_tangents {
                                tangents.push(self.tangents[old]);
                            }

                            if has_uvs {
                                uvs.push(self.uvs[old]);
                            }

                            index
                        };

                        indices.push(new_index);
                    }
                }

                Self {
                    positions,
                    normals,
                    tangents,
                    uvs,
                    indices,
                    material: self.material.clone(),
                    uv_transform: UvTransform::default(),
                }
            })
            .collect()
    }
}

pub fn split_scene_by_connected_faces(scene: &mut MeshScene) {
    for object in &mut scene.objects {
        let mut primitives = Vec::new();

        for primitive in object.primitives.drain(..) {
            if primitive.indices.is_empty() {
                primitives.push(primitive);
                continue;
            }

            let components = primitive.split_by_connected_faces();

            if components.len() <= 1 {
                primitives.push(primitive);
            } else {
                primitives.extend(components);
            }
        }

        object.primitives = primitives;
    }
}
