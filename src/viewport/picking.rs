use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor, WireframeLineWidth},
    prelude::*,
    window::PrimaryWindow,
};

use std::collections::HashMap;

const ISLAND_POSITION_TOLERANCE: f32 = 1.0e-4;
const ISLAND_UV_TOLERANCE: f32 = 1.0e-5;
const PICK_DRAG_THRESHOLD: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UvIslandVertexKey {
    position: [i64; 3],
    uv: [i64; 2],
}

use crate::{
    data::Atlas,
    mesh::{ImportedMeshPart, MeshSceneState},
    ui::EditorUi,
    viewport::TransformMode,
};

use super::{MeshPreview, MeshPreviewCamera, MeshViewportInput, MeshViewportLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshFaceRef {
    pub object_index: usize,
    pub primitive_index: usize,
    pub face_index: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MeshUvIslandSelection {
    pub mesh_id: u64,
    pub faces: Vec<MeshFaceRef>,
}

#[derive(Resource, Default)]
pub struct MeshViewportPicking {
    pressed_position: Option<Vec2>,
    dragged: bool,
    pub selected_uv_island: Option<MeshUvIslandSelection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshPrimitiveSelection {
    pub mesh_id: u64,
    pub object_index: usize,
    pub primitive_index: usize,
}

#[derive(Resource, Default)]
pub struct MeshViewportContextMenu {
    pub selection: Option<MeshPrimitiveSelection>,
    pub position: Vec2,
    pub open: bool,
}

#[derive(Component)]
pub struct MeshUvIslandHighlight;

fn quantize(value: f32, tolerance: f32) -> i64 {
    (value as f64 / tolerance as f64).round() as i64
}

fn island_vertex_key(position: [f32; 3], uv: [f32; 2]) -> Option<UvIslandVertexKey> {
    if !position.iter().chain(uv.iter()).all(|v| v.is_finite()) {
        return None;
    }

    Some(UvIslandVertexKey {
        position: position.map(|v| quantize(v, ISLAND_POSITION_TOLERANCE)),
        uv: uv.map(|v| quantize(v, ISLAND_UV_TOLERANCE)),
    })
}

fn uv_island_for_face(
    scene: &crate::mesh::import::MeshScene,
    object_index: usize,
    primitive_index: usize,
    face_index: usize,
) -> Vec<MeshFaceRef> {
    let Some(object) = scene.objects.get(object_index) else {
        return Vec::new();
    };

    type EdgeKey = (UvIslandVertexKey, UvIslandVertexKey);

    let mut edge_faces: HashMap<EdgeKey, Vec<MeshFaceRef>> = HashMap::new();

    for (primitive_index, primitive) in object.primitives.iter().enumerate() {
        if primitive.uvs.len() != primitive.positions.len() {
            continue;
        }

        for (face_index, triangle) in primitive.indices.chunks_exact(3).enumerate() {
            let mut vertices = [None; 3];

            for corner in 0..3 {
                let index = triangle[corner] as usize;

                let (Some(position), Some(uv)) =
                    (primitive.positions.get(index), primitive.uvs.get(index))
                else {
                    continue;
                };

                vertices[corner] = island_vertex_key(*position, *uv);
            }

            let [Some(a), Some(b), Some(c)] = vertices else {
                continue;
            };

            let face = MeshFaceRef {
                object_index,
                primitive_index,
                face_index,
            };

            for (start, end) in [(a, b), (b, c), (c, a)] {
                if start == end {
                    continue;
                }

                let edge = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };

                edge_faces.entry(edge).or_default().push(face);
            }
        }
    }

    let target = MeshFaceRef {
        object_index,
        primitive_index,
        face_index,
    };

    let mut adjacency: HashMap<MeshFaceRef, Vec<MeshFaceRef>> = HashMap::new();

    for faces in edge_faces.values() {
        for i in 0..faces.len() {
            for j in (i + 1)..faces.len() {
                adjacency.entry(faces[i]).or_default().push(faces[j]);
                adjacency.entry(faces[j]).or_default().push(faces[i]);
            }
        }
    }

    if !adjacency.contains_key(&target) {
        // No matching UV edges: still select the clicked face.
        return vec![target];
    }

    let mut visited = std::collections::HashSet::new();
    let mut stack = vec![target];
    visited.insert(target);

    while let Some(face) = stack.pop() {
        if let Some(neighbors) = adjacency.get(&face) {
            for &neighbor in neighbors {
                if visited.insert(neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }

    let mut result: Vec<_> = visited.into_iter().collect();
    result.sort_by_key(|face| (face.object_index, face.primitive_index, face.face_index));
    result
}

pub fn pick_mesh_primitive(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MeshPreviewCamera>>,
    viewport: Res<MeshViewportLayout>,
    mut mesh_preview: ResMut<MeshPreview>,
    mut picking: ResMut<MeshViewportPicking>,
    viewport_input: Res<MeshViewportInput>,
    mesh_scene_state: Res<MeshSceneState>,
    mut editor_ui: ResMut<EditorUi>,
    atlas: Res<Atlas>,
    parts: Query<(&ImportedMeshPart, &GlobalTransform, &Mesh3d)>,
    meshes: Res<Assets<Mesh>>,
    mut context_menu: ResMut<MeshViewportContextMenu>,
) {
    if viewport_input.pointer_over_ui || viewport_input.floating_window_dragging {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let viewport_min = Vec2::new(viewport.x, viewport.y);
    let viewport_max = viewport_min + Vec2::new(viewport.width, viewport.height);

    if cursor.x < viewport_min.x
        || cursor.y < viewport_min.y
        || cursor.x > viewport_max.x
        || cursor.y > viewport_max.y
    {
        picking.pressed_position = None;
        picking.dragged = false;
        return;
    }

    if buttons.just_pressed(MouseButton::Left) {
        picking.pressed_position = Some(cursor);
        picking.dragged = false;
    }

    if let Some(start) = picking.pressed_position {
        if buttons.pressed(MouseButton::Left) && start.distance(cursor) > PICK_DRAG_THRESHOLD {
            picking.dragged = true;
        }
    }

    let right_clicked = buttons.just_pressed(MouseButton::Right);
    let left_released = buttons.just_released(MouseButton::Left);

    if !left_released && !right_clicked {
        return;
    }

    // Left-click selection is only valid when it wasn't a camera drag.
    if left_released {
        let Some(start) = picking.pressed_position.take() else {
            return;
        };

        let was_dragged = picking.dragged || start.distance(cursor) > PICK_DRAG_THRESHOLD;
        picking.dragged = false;

        if was_dragged {
            return;
        }
    }

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let viewport_position = cursor - viewport_min;

    let Ok(ray) = camera.viewport_to_world(camera_transform, viewport_position) else {
        return;
    };

    let mut closest: Option<(f32, MeshPrimitiveSelection, usize, Vec3, Vec3)> = None;

    for (part, transform, mesh_handle) in &parts {
        let Some(mesh_entry) = mesh_scene_state.get(part.mesh_id) else {
            continue;
        };

        // Hidden meshes must not intercept picking rays.
        if !mesh_entry.visible {
            continue;
        }

        let Some(mesh_part) = mesh_entry
            .parts
            .iter()
            .find(|entry| entry.index == part.part_index)
        else {
            continue;
        };

        // Hidden primitives must not intercept picking rays.
        if !mesh_part.visible {
            continue;
        }

        let Some(mesh) = meshes.get(mesh_handle) else {
            continue;
        };

        let Some((distance, face_index)) = ray_mesh_intersection(&ray, mesh, transform) else {
            continue;
        };

        let selection = MeshPrimitiveSelection {
            mesh_id: part.mesh_id,
            object_index: mesh_part.object_index,
            primitive_index: mesh_part.primitive_index,
        };

        let Some(center) = mesh_world_bounds_center(mesh, transform) else {
            continue;
        };

        let hit_point = ray.origin + *ray.direction * distance;

        let Some(normal) = mesh_world_local_normal(mesh, transform, hit_point, *ray.direction)
        else {
            continue;
        };

        if closest
            .as_ref()
            .is_none_or(|(closest_distance, _, _, _, _)| distance < *closest_distance)
        {
            closest = Some((distance, selection, face_index, center, normal));
        }
    }

    if right_clicked {
        context_menu.position = cursor;
        context_menu.selection = closest.map(|(_, selection, _, _, _)| selection);
        context_menu.open = context_menu.selection.is_some();
        return;
    }

    let Some((_, selection, face_index, center, normal)) = closest else {
        mesh_preview.selected_primitive = None;
        mesh_preview.selected_primitive_center = None;
        mesh_preview.selected_primitive_normal = None;
        picking.selected_uv_island = None;

        if mesh_preview.transform_mode == TransformMode::Uv {
            editor_ui.selected_uv_assignment = None;
        }

        return;
    };

    picking.selected_uv_island =
        mesh_scene_state
            .get(selection.mesh_id)
            .map(|mesh_entry| MeshUvIslandSelection {
                mesh_id: selection.mesh_id,
                faces: uv_island_for_face(
                    &mesh_entry.scene,
                    selection.object_index,
                    selection.primitive_index,
                    face_index,
                ),
            });

    mesh_preview.selected_primitive = Some(selection);
    mesh_preview.selected_primitive_center = Some(center);
    mesh_preview.selected_primitive_normal = Some(normal);

    mesh_preview.selected_primitive = Some(selection);
    mesh_preview.selected_primitive_center = Some(center);

    let material_index = atlas.materials.iter().find_map(|material| {
        material
            .mesh_assignments
            .iter()
            .any(|assignment| {
                assignment.mesh_id == selection.mesh_id
                    && assignment.object_index == selection.object_index
                    && assignment.primitive_index == selection.primitive_index
            })
            .then_some(material.index)
    });

    editor_ui.inspected_atlas_region = material_index;

    if mesh_preview.transform_mode == TransformMode::Uv {
        editor_ui.selected_uv_assignment = Some((
            selection.mesh_id,
            selection.object_index,
            selection.primitive_index,
        ));
    } else {
        editor_ui.selected_uv_assignment = None;
    }
}

fn ray_mesh_intersection(
    ray: &Ray3d,
    mesh: &Mesh,
    transform: &GlobalTransform,
) -> Option<(f32, usize)> {
    use bevy::mesh::VertexAttributeValues;

    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION)? {
        VertexAttributeValues::Float32x3(values) => values,
        _ => return None,
    };

    if mesh.primitive_topology() != bevy::mesh::PrimitiveTopology::TriangleList {
        return None;
    }

    let indices: Vec<usize> = match mesh.indices() {
        Some(indices) => indices.iter().collect(),
        None => (0..positions.len()).collect(),
    };

    let inverse = transform.affine().inverse();
    let local_origin = inverse.transform_point3(ray.origin);
    let local_direction = inverse.transform_vector3(*ray.direction).normalize();

    let mut closest: Option<(f32, usize)> = None;

    for (face_index, triangle) in indices.chunks_exact(3).enumerate() {
        let Some(&a) = positions.get(triangle[0] as usize) else {
            continue;
        };
        let Some(&b) = positions.get(triangle[1] as usize) else {
            continue;
        };
        let Some(&c) = positions.get(triangle[2] as usize) else {
            continue;
        };

        let Some(distance) = ray_triangle_intersection(
            local_origin,
            local_direction,
            Vec3::from_array(a),
            Vec3::from_array(b),
            Vec3::from_array(c),
        ) else {
            continue;
        };

        if closest.is_none_or(|(current, _)| distance < current) {
            closest = Some((distance, face_index));
        }
    }

    closest
}

fn ray_triangle_intersection(
    origin: Vec3,
    direction: Vec3,
    a: Vec3,
    b: Vec3,
    c: Vec3,
) -> Option<f32> {
    // Möller–Trumbore ray/triangle intersection.
    let edge1 = b - a;
    let edge2 = c - a;

    let h = direction.cross(edge2);
    let determinant = edge1.dot(h);

    const EPSILON: f32 = 1e-6;

    if determinant.abs() < EPSILON {
        return None;
    }

    let inverse_determinant = 1.0 / determinant;

    let s = origin - a;
    let u = inverse_determinant * s.dot(h);

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(edge1);
    let v = inverse_determinant * direction.dot(q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let distance = inverse_determinant * edge2.dot(q);

    if distance >= 0.0 {
        Some(distance)
    } else {
        None
    }
}

pub fn sync_mesh_primitive_wireframe(
    picking: Res<MeshViewportPicking>,
    editor_ui: Res<EditorUi>,
    mesh_scene_state: Res<MeshSceneState>,
    mut commands: Commands,
    parts: Query<(Entity, &ImportedMeshPart, &Mesh3d)>,
    old_highlights: Query<Entity, With<MeshUvIslandHighlight>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last_selection: Local<Option<MeshUvIslandSelection>>,
    mut last_style: Local<Option<(bool, [u32; 4], u32)>>,
    mut highlight_material: Local<Option<Handle<StandardMaterial>>>,
) {
    let highlight = &editor_ui.project_settings.highlights.mesh;
    let style = (
        highlight.enabled,
        highlight.color.map(f32::to_bits),
        highlight.strength.to_bits(),
    );

    if *last_selection == picking.selected_uv_island && *last_style == Some(style) {
        return;
    }

    *last_selection = picking.selected_uv_island.clone();
    *last_style = Some(style);

    // Remove previous island overlays.
    for entity in &old_highlights {
        commands.entity(entity).despawn();
    }

    // Remove any legacy whole-primitive wireframes.
    for (entity, _, _) in &parts {
        commands
            .entity(entity)
            .remove::<(Wireframe, WireframeColor, WireframeLineWidth)>();
    }

    if !highlight.enabled {
        return;
    }

    let Some(selection) = picking.selected_uv_island.as_ref() else {
        return;
    };

    if selection.faces.is_empty() {
        return;
    }

    let material = highlight_material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::NONE,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                depth_bias: 0.01,
                ..default()
            })
        })
        .clone();

    // Group selected faces by their original primitive.
    let mut faces_by_primitive: HashMap<(usize, usize), Vec<usize>> = HashMap::new();

    for face in &selection.faces {
        faces_by_primitive
            .entry((face.object_index, face.primitive_index))
            .or_default()
            .push(face.face_index);
    }

    for (entity, part, mesh_handle) in &parts {
        if part.mesh_id != selection.mesh_id {
            continue;
        }

        let Some(mesh_entry) = mesh_scene_state.get(part.mesh_id) else {
            continue;
        };

        let Some(mesh_part) = mesh_entry
            .parts
            .get(part.part_index)
            .filter(|entry| entry.index == part.part_index)
        else {
            continue;
        };

        let Some(face_indices) =
            faces_by_primitive.get(&(mesh_part.object_index, mesh_part.primitive_index))
        else {
            continue;
        };

        let Some(source_mesh) = meshes.get(&mesh_handle.0) else {
            continue;
        };

        if source_mesh.primitive_topology() != bevy::mesh::PrimitiveTopology::TriangleList {
            continue;
        }

        use bevy::mesh::Indices;

        let source_indices: Vec<usize> = match source_mesh.indices() {
            Some(indices) => indices.iter().collect(),
            None => {
                let Some(positions) = source_mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
                    continue;
                };

                let bevy::mesh::VertexAttributeValues::Float32x3(positions) = positions else {
                    continue;
                };

                (0..positions.len()).collect()
            }
        };

        let mut selected_indices = Vec::new();

        for &face_index in face_indices {
            let start = face_index * 3;
            let Some(triangle) = source_indices.get(start..start + 3) else {
                continue;
            };

            selected_indices.extend(triangle.iter().map(|&index| index as u32));
        }

        if selected_indices.is_empty() {
            continue;
        }

        // Clone the source mesh and retain only the selected triangles.
        let mut overlay_mesh = source_mesh.clone();
        overlay_mesh.insert_indices(Indices::U32(selected_indices));

        let color = Color::srgba(
            highlight.color[0],
            highlight.color[1],
            highlight.color[2],
            highlight.color[3] * highlight.strength,
        );

        commands.spawn((
            Mesh3d(meshes.add(overlay_mesh)),
            MeshMaterial3d(material.clone()),
            Wireframe,
            WireframeColor { color },
            WireframeLineWidth { width: 1.0 },
            MeshUvIslandHighlight,
            ChildOf(entity),
            Transform::default(),
        ));
    }
}

fn mesh_world_bounds_center(mesh: &Mesh, transform: &GlobalTransform) -> Option<Vec3> {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION)? {
        bevy::mesh::VertexAttributeValues::Float32x3(values) => values,
        _ => return None,
    };

    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for position in positions {
        let world_position = transform.transform_point(Vec3::from_array(*position));

        min = min.min(world_position);
        max = max.max(world_position);
    }

    (min.is_finite() && max.is_finite()).then_some((min + max) * 0.5)
}

fn mesh_world_local_normal(
    mesh: &Mesh,
    transform: &GlobalTransform,
    hit_point: Vec3,
    ray_direction: Vec3,
) -> Option<Vec3> {
    use bevy::mesh::{Indices, VertexAttributeValues};

    if mesh.primitive_topology() != bevy::mesh::PrimitiveTopology::TriangleList {
        return None;
    }

    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION)? {
        VertexAttributeValues::Float32x3(values) => values,
        _ => return None,
    };

    let indices: Vec<usize> = match mesh.indices() {
        Some(Indices::U16(values)) => values.iter().map(|&i| i as usize).collect(),
        Some(Indices::U32(values)) => values.iter().map(|&i| i as usize).collect(),
        None => (0..positions.len()).collect(),
    };

    // (distance squared from hit, normal, triangle area)
    let mut triangles = Vec::new();
    let mut bounds_min = Vec3::splat(f32::INFINITY);
    let mut bounds_max = Vec3::splat(f32::NEG_INFINITY);

    for triangle in indices.chunks_exact(3) {
        let Some(&pa) = positions.get(triangle[0]) else {
            continue;
        };
        let Some(&pb) = positions.get(triangle[1]) else {
            continue;
        };
        let Some(&pc) = positions.get(triangle[2]) else {
            continue;
        };

        let a = transform.transform_point(Vec3::from_array(pa));
        let b = transform.transform_point(Vec3::from_array(pb));
        let c = transform.transform_point(Vec3::from_array(pc));

        bounds_min = bounds_min.min(a).min(b).min(c);
        bounds_max = bounds_max.max(a).max(b).max(c);

        let cross = (b - a).cross(c - a);
        let double_area = cross.length();

        if double_area <= 1e-8 {
            continue;
        }

        let centroid = (a + b + c) / 3.0;
        let mut normal = cross / double_area;

        // Orient the normal toward the side from which the ray arrived.
        if normal.dot(ray_direction) > 0.0 {
            normal = -normal;
        }

        triangles.push((
            centroid.distance_squared(hit_point),
            normal,
            double_area * 0.5,
        ));
    }

    if triangles.is_empty() {
        return None;
    }

    // Use the closest triangle as the local reference direction.
    triangles.sort_by(|a, b| a.0.total_cmp(&b.0));
    let reference = triangles[0].1;

    let bounds_radius = (bounds_max - bounds_min).length().max(1e-4);
    let neighborhood_radius = bounds_radius * 0.15;
    let radius_squared = neighborhood_radius * neighborhood_radius;

    let mut normal_sum = Vec3::ZERO;

    // Only use a small local neighborhood, not the entire primitive.
    for &(distance_squared, normal, area) in triangles.iter().take(16) {
        // Ignore triangles facing a substantially different direction.
        if normal.dot(reference) < 0.25 {
            continue;
        }

        // Nearby triangles contribute more than distant ones.
        let distance_weight = 1.0 / (1.0 + distance_squared / radius_squared);
        normal_sum += normal * area * distance_weight;
    }

    if normal_sum.length_squared() > 1e-12 {
        Some(normal_sum.normalize())
    } else {
        Some(reference)
    }
}
