use std::path::Path;

use anyhow::{Result, anyhow};
use bevy::prelude::*;

mod exr;
mod standard;

pub fn load(path: &Path) -> Result<Image> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    match extension.as_deref() {
        Some("exr") => exr::load_exr(path),

        Some("png" | "jpg" | "jpeg" | "tga" | "bmp" | "tif" | "tiff") => {
            standard::load_standard(path)
        }

        Some(ext) => Err(anyhow!("unsupported image format: .{ext}")),
        None => Err(anyhow!("image has no file extension")),
    }
}

pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "exr" | "png" | "jpg" | "jpeg" | "tga" | "bmp" | "tif" | "tiff"
            )
        })
}
