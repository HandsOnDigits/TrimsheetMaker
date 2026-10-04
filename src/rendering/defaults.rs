use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefaultTexture {
    Checkerboard,
    White,
    Black,
    FlatNormal,
}

#[derive(Resource)]
pub struct DefaultTextures {
    pub checkerboard: Handle<Image>,
    pub white: Handle<Image>,
    pub black: Handle<Image>,
    pub flat_normal: Handle<Image>,
}

impl DefaultTextures {
    pub fn create(images: &mut Assets<Image>) -> Self {
        Self {
            checkerboard: images.add(checkerboard()),
            white: images.add(solid_color(
                [255, 255, 255, 255],
                TextureFormat::Rgba8UnormSrgb,
            )),
            black: images.add(solid_color([0, 0, 0, 255], TextureFormat::Rgba8Unorm)),
            flat_normal: images.add(solid_color([128, 128, 255, 255], TextureFormat::Rgba8Unorm)),
        }
    }

    pub fn get(&self, texture: DefaultTexture) -> &Handle<Image> {
        match texture {
            DefaultTexture::Checkerboard => &self.checkerboard,
            DefaultTexture::White => &self.white,
            DefaultTexture::Black => &self.black,
            DefaultTexture::FlatNormal => &self.flat_normal,
        }
    }
}

fn solid_color(color: [u8; 4], format: TextureFormat) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &color,
        format,
        RenderAssetUsages::default(),
    );

    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;

    image
}

fn checkerboard() -> Image {
    const SIZE: u32 = 64;
    const TILE: u32 = 8;

    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let value = if ((x / TILE) + (y / TILE)) % 2 == 0 {
                70
            } else {
                150
            };

            data.extend_from_slice(&[value, value, value, 255]);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;

    image
}
