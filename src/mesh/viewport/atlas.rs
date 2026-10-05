use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use futures_lite::future;

use crate::{
    data::{Atlas, AtlasBuildResult, build_atlas, create_atlas_build_input},
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
