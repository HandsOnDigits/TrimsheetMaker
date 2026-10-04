use std::path::Path;

use anyhow::Result;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub fn load_standard(path: &Path) -> Result<Image> {
    let image = ::image::open(path)?.to_rgba8();

    let (width, height) = image.dimensions();

    Ok(Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        image.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}
