use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor, WireframeLineWidth},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    data::Atlas,
    mesh::{ImportedMeshPart, MeshSceneState},
    ui::EditorUi,
    viewport::TransformMode,
};

use super::{MeshPreview, MeshPreviewCamera, MeshViewportInput, MeshViewportLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshPrimitiveSelection {
    pub mesh_id: u64,
    pub object_index: usize,
    pub primitive_index: usize,
}

#[derive(Resource, Default)]
pub struct MeshViewportPicking {
    pressed_position: Option<Vec2>,
    dragged: bool,
}

const PICK_DRAG_THRESHOLD: f32 = 4.0;

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

    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    let Some(start) = picking.pressed_position.take() else {
        return;
    };

    let was_dragged = picking.dragged || start.distance(cursor) > PICK_DRAG_THRESHOLD;

    picking.dragged = false;

    // A drag belongs to the camera, never to selection.
    if was_dragged {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let viewport_position = cursor - viewport_min;

    let Ok(ray) = camera.viewport_to_world(camera_transform, viewport_position) else {
        return;
    };

    let mut closest: Option<(f32, MeshPrimitiveSelection)> = None;

    for (part, transform, mesh_handle) in &parts {
        let Some(mesh_entry) = mesh_scene_state.get(part.mesh_id) else {
            continue;
        };

        let Some(mesh_part) = mesh_entry
            .parts
            .iter()
            .find(|entry| entry.index == part.part_index)
        else {
            continue;
        };

        let Some(mesh) = meshes.get(mesh_handle) else {
            continue;
        };

        let Some(distance) = ray_mesh_intersection(&ray, mesh, transform) else {
            continue;
        };

        let selection = MeshPrimitiveSelection {
            mesh_id: part.mesh_id,
            object_index: mesh_part.object_index,
            primitive_index: mesh_part.primitive_index,
        };

        if closest
            .as_ref()
            .is_none_or(|(closest_distance, _)| distance < *closest_distance)
        {
            closest = Some((distance, selection));
        }
    }

    let Some((_, selection)) = closest else {
        mesh_preview.selected_primitive = None;

        if mesh_preview.transform_mode == TransformMode::Uv {
            editor_ui.selected_uv_assignment = None;
        }

        return;
    };

    mesh_preview.selected_primitive = Some(selection);

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

fn ray_mesh_intersection(ray: &Ray3d, mesh: &Mesh, transform: &GlobalTransform) -> Option<f32> {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION)? {
        bevy::mesh::VertexAttributeValues::Float32x3(values) => values,
        _ => return None,
    };

    let indices = mesh.indices()?;

    let inverse = transform.affine().inverse();

    let local_origin = inverse.transform_point3(ray.origin);
    let local_direction = inverse.transform_vector3(*ray.direction).normalize();

    let mut closest = None;

    let mut triangle_indices = indices.iter();

    loop {
        let Some(i0) = triangle_indices.next() else {
            break;
        };

        let Some(i1) = triangle_indices.next() else {
            break;
        };

        let Some(i2) = triangle_indices.next() else {
            break;
        };

        let Some(a) = positions.get(i0 as usize) else {
            continue;
        };

        let Some(b) = positions.get(i1 as usize) else {
            continue;
        };

        let Some(c) = positions.get(i2 as usize) else {
            continue;
        };

        let Some(distance) = ray_triangle_intersection(
            local_origin,
            local_direction,
            Vec3::from_array(*a),
            Vec3::from_array(*b),
            Vec3::from_array(*c),
        ) else {
            continue;
        };

        if closest.is_none_or(|current| distance < current) {
            closest = Some(distance);
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
    mesh_preview: Res<MeshPreview>,
    mesh_scene_state: Res<MeshSceneState>,
    editor_ui: Res<EditorUi>,
    mut commands: Commands,
    parts: Query<(Entity, &ImportedMeshPart)>,
) {
    let selected = mesh_preview.selected_primitive;

    for (entity, part) in &parts {
        let is_selected = selected.is_some_and(|selected| {
            if selected.mesh_id != part.mesh_id {
                return false;
            }

            let Some(mesh_entry) = mesh_scene_state.get(part.mesh_id) else {
                return false;
            };

            let Some(mesh_part) = mesh_entry
                .parts
                .iter()
                .find(|entry| entry.index == part.part_index)
            else {
                return false;
            };

            selected.object_index == mesh_part.object_index
                && selected.primitive_index == mesh_part.primitive_index
        });

        if is_selected && editor_ui.project_settings.highlights.mesh.enabled {
            let highlight = &editor_ui.project_settings.highlights.mesh;

            commands.entity(entity).insert((
                Wireframe,
                WireframeColor {
                    color: Color::srgba(
                        highlight.color[0],
                        highlight.color[1],
                        highlight.color[2],
                        highlight.color[3] * highlight.strength,
                    ),
                },
                WireframeLineWidth { width: 1.0 },
            ));
        } else {
            commands
                .entity(entity)
                .remove::<(Wireframe, WireframeColor, WireframeLineWidth)>();
        }
    }
}
