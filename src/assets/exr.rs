use std::path::Path;

use anyhow::{Result, anyhow};

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub fn load_exr(path: &Path) -> Result<Image> {
    let image = exr::prelude::read_first_flat_layer_from_file(path)?;

    let width = image.layer_data.size.x();
    let height = image.layer_data.size.y();

    let channels = &image.layer_data.channel_data.list;

    let channel = |name: &str| {
        channels
            .iter()
            .find(|channel| channel.name.to_string().eq_ignore_ascii_case(name))
    };

    let red = channel("R");
    let green = channel("G");
    let blue = channel("B");
    let alpha = channel("A");
    let luminance = channel("Y");

    let channel_count = channels.len();

    let (format, data) = match (red, green, blue, alpha, luminance, channel_count) {
        (Some(r), Some(g), Some(b), a, _, _) => {
            let mut data = Vec::with_capacity(width * height * 16);

            for index in 0..(width * height) {
                let r = r.sample_data.value_by_flat_index(index).to_f32();
                let g = g.sample_data.value_by_flat_index(index).to_f32();
                let b = b.sample_data.value_by_flat_index(index).to_f32();

                let a = a
                    .map(|channel| channel.sample_data.value_by_flat_index(index).to_f32())
                    .unwrap_or(1.0);

                data.extend_from_slice(&r.to_le_bytes());
                data.extend_from_slice(&g.to_le_bytes());
                data.extend_from_slice(&b.to_le_bytes());
                data.extend_from_slice(&a.to_le_bytes());
            }

            (TextureFormat::Rgba32Float, data)
        }

        (_, _, _, _, Some(y), 1) => {
            let mut data = Vec::with_capacity(width * height * 4);

            for index in 0..(width * height) {
                let value = y.sample_data.value_by_flat_index(index).to_f32();

                data.extend_from_slice(&value.to_le_bytes());
            }

            (TextureFormat::R32Float, data)
        }

        (_, _, _, _, _, 1) => {
            let channel = &channels[0];

            let mut data = Vec::with_capacity(width * height * 4);

            for index in 0..(width * height) {
                let value = channel.sample_data.value_by_flat_index(index).to_f32();

                data.extend_from_slice(&value.to_le_bytes());
            }

            (TextureFormat::R32Float, data)
        }

        _ => {
            return Err(anyhow!(
                "unsupported EXR channel layout ({} channels)",
                channel_count
            ));
        }
    };

    Ok(Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        format,
        RenderAssetUsages::default(),
    ))
}
