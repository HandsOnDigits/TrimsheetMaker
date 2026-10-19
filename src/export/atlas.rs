use bevy::{prelude::*, render::render_resource::TextureFormat};

use crate::data::{Atlas, AtlasMap, atlas_source_image_from_image, read_source_pixel};

pub fn export_atlas_maps(atlas: &Atlas, images: &Assets<Image>) -> anyhow::Result<usize> {
    use exr::prelude::write_rgba_file;
    use std::path::PathBuf;

    let Some(output_dir) = rfd::FileDialog::new().pick_folder() else {
        return Ok(0);
    };

    let maps = [
        (AtlasMap::BaseColor, "atlas_base_color.exr"),
        (AtlasMap::Normal, "atlas_normal.exr"),
        (AtlasMap::Height, "atlas_height.exr"),
        (AtlasMap::AmbientOcclusion, "atlas_ao.exr"),
        (AtlasMap::Roughness, "atlas_roughness.exr"),
        (AtlasMap::Metallic, "atlas_metallic.exr"),
    ];

    let mut exported = 0;

    for (map, filename) in maps {
        let Some(handle) = atlas.images.get(map) else {
            continue;
        };

        let Some(image) = images.get(handle) else {
            anyhow::bail!("Atlas image for '{}' is not available.", filename);
        };

        let Some(source) = atlas_source_image_from_image(image) else {
            anyhow::bail!(
                "Atlas image '{}' has no CPU-accessible pixel data.",
                filename
            );
        };

        let path: PathBuf = output_dir.join(filename);

        write_rgba_file(&path, source.width, source.height, |x, y| {
            let pixel =
                read_source_pixel(&source, x, y).expect("Unsupported or invalid atlas pixel data");

            let [mut r, mut g, mut b, a] = pixel;

            // Base Color is stored as sRGB in the generated atlas.
            // Convert to linear values for the EXR output.
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

        exported += 1;
    }

    if exported == 0 {
        anyhow::bail!("There are no generated atlas maps to export.");
    }

    Ok(exported)
}
