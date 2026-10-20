use bevy::{prelude::*, render::render_resource::TextureFormat};

use std::path::PathBuf;

use crate::data::{
    Atlas, AtlasMap, AtlasSourceImage, atlas_source_image_from_image, read_source_pixel,
};

pub fn start_atlas_export(
    task: &mut super::task::ExportTaskState,
    atlas: &Atlas,
    images: &Assets<Image>,
) -> anyhow::Result<bool> {
    if task.is_active() {
        return Ok(false);
    }

    let Some(output_dir) = rfd::FileDialog::new().pick_folder() else {
        info!("Export Atlas cancelled");
        return Ok(false);
    };

    let maps = [
        (AtlasMap::BaseColor, "atlas_base_color.exr"),
        (AtlasMap::Normal, "atlas_normal.exr"),
        (AtlasMap::Height, "atlas_height.exr"),
        (AtlasMap::AmbientOcclusion, "atlas_ao.exr"),
        (AtlasMap::Roughness, "atlas_roughness.exr"),
        (AtlasMap::Metallic, "atlas_metallic.exr"),
    ];

    // Snapshot all required pixel data before starting the worker.
    let mut snapshots: Vec<(AtlasMap, PathBuf, AtlasSourceImage)> = Vec::new();

    for (map, filename) in maps {
        let Some(handle) = atlas.images.get(map) else {
            continue;
        };

        let Some(image) = images.get(handle) else {
            anyhow::bail!("Atlas image for '{filename}' is not available.");
        };

        let Some(source) = atlas_source_image_from_image(image) else {
            anyhow::bail!("Atlas image '{filename}' has no CPU-accessible pixel data.");
        };

        snapshots.push((map, output_dir.join(filename), source));
    }

    if snapshots.is_empty() {
        anyhow::bail!("There are no generated atlas maps to export.");
    }

    task.start("Exporting atlas textures…", move |sender| {
        use exr::prelude::write_rgba_file;

        let total = snapshots.len();

        for (index, (map, path, source)) in snapshots.into_iter().enumerate() {
            write_rgba_file(&path, source.width, source.height, |x, y| {
                let [mut r, mut g, mut b, a] = read_source_pixel(&source, x, y)
                    .expect("Unsupported or invalid atlas pixel data");

                if map == AtlasMap::BaseColor && source.format == TextureFormat::Rgba8UnormSrgb {
                    let to_linear = |v: f32| {
                        if v <= 0.04045 {
                            v / 12.92
                        } else {
                            ((v + 0.055) / 1.055).powf(2.4)
                        }
                    };

                    r = to_linear(r);
                    g = to_linear(g);
                    b = to_linear(b);
                }

                (r, g, b, a)
            })
            .map_err(|error| anyhow::anyhow!("Failed to export '{}': {error}", path.display()))?;

            let progress = (index + 1) as f32 / total as f32;
            let _ = sender.send(super::task::ExportMessage::Progress(progress));
        }

        Ok(format!("Exported {total} atlas texture maps"))
    })
    .then_some(true)
    .ok_or_else(|| anyhow::anyhow!("An export is already in progress"))
}
