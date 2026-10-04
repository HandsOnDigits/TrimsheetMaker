use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use futures_lite::future;

use crate::{
    data::{Atlas, AtlasBuildResult, AtlasMaterial, build_atlas, create_atlas_build_input},
    transform::Rotation,
    ui::EditorUi,
};

#[derive(Resource)]
pub struct AtlasRefresh {
    pub requested: bool,
    pub task: Option<Task<AtlasBuildResult>>,
    pub generation: u64,

    pub building_image_revision: u64,
    pub building_bake_revision: u64,
    pub building_rendering_revision: u64,
}

impl Default for AtlasRefresh {
    fn default() -> Self {
        Self {
            requested: true,
            task: None,
            generation: 0,
            building_image_revision: u64::MAX,
            building_bake_revision: u64::MAX,
            building_rendering_revision: u64::MAX,
        }
    }
}

pub fn sync_atlas_images(
    atlas: Res<Atlas>,
    editor_ui: Res<EditorUi>,
    images: Res<Assets<Image>>,
    mut refresh: ResMut<AtlasRefresh>,
) {
    if atlas.image_revision != refresh.building_image_revision
        || atlas.bake_revision != refresh.building_bake_revision
        || editor_ui.rendering_revision != refresh.building_rendering_revision
    {
        refresh.requested = true;
    }

    if !refresh.requested || refresh.task.is_some() {
        return;
    }

    let image_revision = atlas.image_revision;
    let bake_revision = atlas.bake_revision;
    let rendering_revision = editor_ui.rendering_revision;

    let input =
        create_atlas_build_input(&atlas, &editor_ui.project_settings.texture_naming, &images);

    refresh.generation = refresh.generation.wrapping_add(1);

    refresh.building_image_revision = image_revision;
    refresh.building_bake_revision = bake_revision;
    refresh.building_rendering_revision = rendering_revision;

    let generation = refresh.generation;

    refresh.task =
        Some(AsyncComputeTaskPool::get().spawn(async move { build_atlas(input, generation) }));
}

pub fn finish_atlas_build(
    mut atlas: ResMut<Atlas>,
    mut images: ResMut<Assets<Image>>,
    mut refresh: ResMut<AtlasRefresh>,
    editor_ui: Res<EditorUi>,
) {
    let Some(mut task) = refresh.task.take() else {
        return;
    };

    let Some(result) = future::block_on(future::poll_once(&mut task)) else {
        refresh.task = Some(task);
        return;
    };

    if result.generation != refresh.generation {
        return;
    }

    for (map, image) in result.maps {
        match atlas.images.get(map).cloned() {
            Some(handle) => {
                if let Some(mut existing) = images.get_mut(&handle) {
                    *existing = image;
                } else {
                    let new_handle = images.add(image);
                    *atlas.images.get_mut(map) = Some(new_handle);
                }
            }

            None => {
                let handle = images.add(image);
                *atlas.images.get_mut(map) = Some(handle);
            }
        }
    }

    let still_current = atlas.image_revision == refresh.building_image_revision
        && atlas.bake_revision == refresh.building_bake_revision
        && editor_ui.rendering_revision == refresh.building_rendering_revision;

    refresh.requested = !still_current;
}

pub fn ensure_atlas_build_requested(atlas: Res<Atlas>, mut refresh: ResMut<AtlasRefresh>) {
    if refresh.task.is_some() {
        return;
    }

    if atlas.images.base_color.is_none()
        && atlas.images.normal.is_none()
        && atlas.images.height.is_none()
        && atlas.images.ambient_occlusion.is_none()
        && atlas.images.roughness.is_none()
        && atlas.images.metallic.is_none()
    {
        refresh.requested = true;
    }
}

fn transform_material_uv(uv: [f32; 2], rotation: Rotation) -> [f32; 2] {
    match rotation {
        Rotation::Deg0 => uv,
        Rotation::Deg90 => [uv[1], 1.0 - uv[0]],
        Rotation::Deg180 => [1.0 - uv[0], 1.0 - uv[1]],
        Rotation::Deg270 => [1.0 - uv[1], uv[0]],
    }
}

pub(crate) fn atlas_uv(uv: [f32; 2], material: &AtlasMaterial, atlas: &Atlas) -> [f32; 2] {
    let uv = transform_material_uv(uv, material.transform.rotation);

    let atlas_width = atlas.width as f32;
    let atlas_height = atlas.height as f32;

    if atlas_width <= 0.0 || atlas_height <= 0.0 {
        return [0.0, 0.0];
    }

    let x = material.transform.x as f32;
    let y = material.transform.y as f32;
    let width = material.transform.width as f32;
    let height = material.transform.height as f32;

    [
        (x + uv[0] * width) / atlas_width,
        (y + (1.0 - uv[1]) * height) / atlas_height,
    ]
}

pub(crate) fn material_uvs(
    uvs: &[[f32; 2]],
    material: Option<&AtlasMaterial>,
    atlas: &Atlas,
) -> Vec<[f32; 2]> {
    let Some(material) = material else {
        return vec![[0.0, 0.0]; uvs.len()];
    };

    uvs.iter()
        .copied()
        .map(|uv| atlas_uv(uv, material, atlas))
        .collect()
}
