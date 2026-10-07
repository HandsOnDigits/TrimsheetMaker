use rayon::prelude::*;

use std::{collections::BTreeMap, path::Path, sync::Arc};

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use bevy_egui::egui;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::transform::{Rotation, Transform};

use super::{
    mesh::import::MeshMaterialRef,
    rendering::{Channel, ChannelSource, RenderingMaterialBindings},
};

const ATLAS_ROW_CHUNK: usize = 32;

#[derive(Clone)]
pub struct MaterialTexture {
    pub id: String,

    // Original source file. Never discard this.
    pub path: PathBuf,

    pub group_id: Option<String>,

    // Runtime representation, if loaded.
    pub image: Option<Handle<Image>>,
    pub egui_texture: Option<egui::TextureId>,
}

pub struct MaterialSource {
    pub name: String,
    pub material_ref: Option<MeshMaterialRef>,
    pub textures: Vec<MaterialTexture>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TextureGroup {
    pub id: String,
    pub name: String,
    pub suffixes: Vec<String>,

    #[serde(default)]
    pub rendering: RenderingMaterialBindings,
}

impl Default for TextureGroup {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            suffixes: Vec::new(),
            rendering: RenderingMaterialBindings::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TextureNaming {
    #[serde(default)]
    pub groups: Vec<TextureGroup>,
}

impl TextureNaming {
    pub fn reclassify_atlas(&self, atlas: &mut Atlas) {
        for material in &mut atlas.materials {
            for texture in &mut material.textures {
                texture.group_id = self.group_for_filename(
                    texture
                        .path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default(),
                );
            }
        }
    }

    pub fn reclassify_materials(&self, materials: &mut BTreeMap<String, MaterialSource>) {
        for material in materials.values_mut() {
            for texture in &mut material.textures {
                texture.group_id = self.group_for_filename(
                    texture
                        .path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default(),
                );
            }
        }
    }

    pub fn group_for_filename(&self, filename: &str) -> Option<String> {
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename);

        self.groups
            .iter()
            .flat_map(|group| group.suffixes.iter().map(move |suffix| (group, suffix)))
            .filter(|(_, suffix)| {
                stem.to_ascii_lowercase()
                    .ends_with(&suffix.trim().to_ascii_lowercase())
            })
            .max_by_key(|(_, suffix)| suffix.len())
            .map(|(group, _)| group.id.clone())
    }

    pub fn texture_matches_group(&self, path: &Path, group_id: &str) -> bool {
        let Some(group) = self.groups.iter().find(|group| group.id == group_id) else {
            return false;
        };

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        group
            .suffixes
            .iter()
            .any(|suffix| stem.ends_with(&suffix.trim().to_ascii_lowercase()))
    }

    pub fn merge_detected(&mut self, detected: TextureNaming) {
        for detected_group in detected.groups {
            let mut matched = false;

            for existing_group in &mut self.groups {
                let suffix_matches = detected_group.suffixes.iter().any(|detected_suffix| {
                    existing_group.suffixes.iter().any(|existing_suffix| {
                        existing_suffix.eq_ignore_ascii_case(detected_suffix)
                    })
                });

                if suffix_matches {
                    for suffix in detected_group.suffixes.clone() {
                        if !existing_group
                            .suffixes
                            .iter()
                            .any(|existing| existing.eq_ignore_ascii_case(&suffix))
                        {
                            existing_group.suffixes.push(suffix);
                        }
                    }

                    matched = true;
                    break;
                }
            }

            if !matched {
                self.groups.push(detected_group);
            }
        }
    }
}

impl Default for TextureNaming {
    fn default() -> Self {
        Self {
            groups: vec![
                TextureGroup {
                    id: "base_color".into(),
                    name: "Base Color".into(),
                    suffixes: vec![
                        "_basecolor".into(),
                        "_base_color".into(),
                        "_albedo".into(),
                        "_diffuse".into(),
                    ],
                    rendering: RenderingMaterialBindings {
                        base_color: true,
                        ..default()
                    },
                },
                TextureGroup {
                    id: "normal".into(),
                    name: "Normal".into(),
                    suffixes: vec![
                        "_normal".into(),
                        "_normal_gl".into(),
                        "_normal_dx".into(),
                        "_nrm".into(),
                    ],
                    rendering: RenderingMaterialBindings {
                        normal: true,
                        ..default()
                    },
                },
                TextureGroup {
                    id: "ormh".into(),
                    name: "ORMH".into(),
                    suffixes: vec![
                        "_orm".into(),
                        "_ormh".into(),
                        "_occlusion_roughness_metallic".into(),
                        "_occlusion_roughness_metallic_height".into(),
                    ],
                    rendering: RenderingMaterialBindings {
                        ambient_occlusion: Some(ChannelSource {
                            channel: Channel::Red,
                        }),
                        roughness: Some(ChannelSource {
                            channel: Channel::Green,
                        }),
                        metallic: Some(ChannelSource {
                            channel: Channel::Blue,
                        }),
                        height: Some(ChannelSource {
                            channel: Channel::Alpha,
                        }),
                        ..default()
                    },
                },
                TextureGroup {
                    id: "roughness".into(),
                    name: "Roughness".into(),
                    suffixes: vec!["_roughness".into(), "_rough".into()],
                    rendering: RenderingMaterialBindings {
                        roughness: Some(ChannelSource {
                            channel: Channel::Red,
                        }),
                        ..default()
                    },
                },
                TextureGroup {
                    id: "metallic".into(),
                    name: "Metallic".into(),
                    suffixes: vec!["_metallic".into(), "_metalness".into(), "_metal".into()],
                    rendering: RenderingMaterialBindings {
                        metallic: Some(ChannelSource {
                            channel: Channel::Red,
                        }),
                        ..default()
                    },
                },
                TextureGroup {
                    id: "ao".into(),
                    name: "Ambient Occlusion".into(),
                    suffixes: vec![
                        "_ao".into(),
                        "_occlusion".into(),
                        "_ambient_occlusio".into(),
                        "_ambientocclusio".into(),
                    ],
                    rendering: RenderingMaterialBindings {
                        ambient_occlusion: Some(ChannelSource {
                            channel: Channel::Red,
                        }),
                        ..default()
                    },
                },
                TextureGroup {
                    id: "height".into(),
                    name: "Height".into(),
                    suffixes: vec!["_height".into(), "_displacement".into(), "_disp".into()],
                    rendering: RenderingMaterialBindings {
                        height: Some(ChannelSource {
                            channel: Channel::Red,
                        }),
                        ..default()
                    },
                },
            ],
        }
    }
}
#[derive(Clone)]
pub struct AtlasMaterialBuildInput {
    pub(crate) transform: Transform,
    pub(crate) maps: BTreeMap<AtlasMap, AtlasSourceImage>,
    pub(crate) channel_sources: Vec<(AtlasMap, AtlasSourceImage, Channel)>,
}

pub struct AtlasBuildInput {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) materials: Vec<AtlasMaterialBuildInput>,
}

#[derive(Clone)]
pub(crate) struct AtlasSourceImage {
    pub(crate) format: TextureFormat,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) data: Arc<[u8]>,
}

pub struct AtlasBuildResult {
    pub generation: u64,
    pub maps: Vec<(AtlasMap, Image)>,
}

#[derive(Clone, Copy)]
struct PreparedAtlasMaterial<'a> {
    image: &'a AtlasSourceImage,

    x: i64,
    y: i64,

    width: usize,
    height: usize,

    rotation: Rotation,

    source_width: usize,
    source_height: usize,
}

fn prepare_material<'a>(
    material: &'a AtlasMaterialBuildInput,
    map: AtlasMap,
) -> Option<PreparedAtlasMaterial<'a>> {
    let image = material.maps.get(&map)?;

    let (x, y, width, height) = atlas_destination_rect(material.transform)?;

    Some(PreparedAtlasMaterial {
        image,

        x,
        y,

        width,
        height,

        rotation: material.transform.rotation,

        source_width: image.width,
        source_height: image.height,
    })
}

fn atlas_source_image_from_image(image: &Image) -> Option<AtlasSourceImage> {
    let data = image.data.as_ref()?;

    Some(AtlasSourceImage {
        format: image.texture_descriptor.format,
        width: image.width() as usize,
        height: image.height() as usize,
        data: Arc::from(data.as_slice()),
    })
}

fn texture_source_image(
    texture: &MaterialTexture,
    images: &Assets<Image>,
) -> Option<AtlasSourceImage> {
    let handle = texture.image.as_ref()?;
    let image = images.get(handle)?;

    atlas_source_image_from_image(image)
}

fn read_f32(source_data: &[u8], offset: usize) -> Option<f32> {
    let bytes = source_data.get(offset..offset + 4)?;

    Some(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_f16(source_data: &[u8], offset: usize) -> Option<f32> {
    let bytes = source_data.get(offset..offset + 2)?;
    let bits = u16::from_le_bytes([bytes[0], bytes[1]]);

    Some(half::f16::from_bits(bits).to_f32())
}

fn bake_channel_source(source: &AtlasSourceImage, channel: Channel) -> Option<AtlasSourceImage> {
    let channel_index = match channel {
        Channel::Red => 0,
        Channel::Green => 1,
        Channel::Blue => 2,
        Channel::Alpha => 3,
    };

    let pixel_count = source.width.checked_mul(source.height)?;
    let data_len = pixel_count.checked_mul(16)?;

    let mut output = vec![0u8; data_len];

    output
        .par_chunks_mut(16)
        .enumerate()
        .for_each(|(pixel, output)| {
            let value = match source.format {
                TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => {
                    let offset = pixel * 4 + channel_index;

                    source
                        .data
                        .get(offset)
                        .map(|value| *value as f32 / 255.0)
                        .unwrap_or(0.0)
                }

                TextureFormat::R32Float => {
                    if channel_index != 0 {
                        0.0
                    } else {
                        read_f32(&source.data, pixel * 4).unwrap_or(0.0)
                    }
                }

                TextureFormat::Rgba32Float => {
                    read_f32(&source.data, pixel * 16 + channel_index * 4).unwrap_or(0.0)
                }

                TextureFormat::R16Float => {
                    if channel_index != 0 {
                        0.0
                    } else {
                        read_f16(&source.data, pixel * 2).unwrap_or(0.0)
                    }
                }

                TextureFormat::Rgba16Float => {
                    read_f16(&source.data, pixel * 8 + channel_index * 2).unwrap_or(0.0)
                }

                _ => return,
            };

            let bytes = value.to_le_bytes();

            output[0..4].copy_from_slice(&bytes);
            output[4..8].copy_from_slice(&bytes);
            output[8..12].copy_from_slice(&bytes);

            output[12..16].copy_from_slice(&1.0f32.to_le_bytes());
        });

    Some(AtlasSourceImage {
        format: TextureFormat::Rgba32Float,
        width: source.width,
        height: source.height,
        data: Arc::from(output),
    })
}

fn add_group_rendering_bindings(
    maps: &mut BTreeMap<AtlasMap, AtlasSourceImage>,
    channel_sources: &mut Vec<(AtlasMap, AtlasSourceImage, Channel)>,
    material: &AtlasMaterial,
    texture: &MaterialTexture,
    group: &TextureGroup,
    images: &Assets<Image>,
) {
    let Some(source) = texture_source_image(texture, images) else {
        warn!(
            "Failed to load texture '{}' for material '{}'",
            texture.path.display(),
            material.name
        );
        return;
    };

    let rendering = &group.rendering;

    if rendering.base_color {
        maps.insert(AtlasMap::BaseColor, source.clone());
    }

    if rendering.normal {
        maps.insert(AtlasMap::Normal, source.clone());
    }

    if let Some(binding) = rendering.height.as_ref() {
        channel_sources.push((AtlasMap::Height, source.clone(), binding.channel));
    }

    if let Some(binding) = rendering.ambient_occlusion.as_ref() {
        channel_sources.push((AtlasMap::AmbientOcclusion, source.clone(), binding.channel));
    }

    if let Some(binding) = rendering.roughness.as_ref() {
        channel_sources.push((AtlasMap::Roughness, source.clone(), binding.channel));
    }

    if let Some(binding) = rendering.metallic.as_ref() {
        channel_sources.push((AtlasMap::Metallic, source, binding.channel));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AtlasMap {
    BaseColor,
    Normal,
    Height,
    AmbientOcclusion,
    Roughness,
    Metallic,
}

#[derive(Default)]
pub struct AtlasImages {
    pub base_color: Option<Handle<Image>>,
    pub normal: Option<Handle<Image>>,
    pub height: Option<Handle<Image>>,
    pub ambient_occlusion: Option<Handle<Image>>,
    pub roughness: Option<Handle<Image>>,
    pub metallic: Option<Handle<Image>>,
}

impl AtlasImages {
    pub fn get(&self, map: AtlasMap) -> Option<&Handle<Image>> {
        match map {
            AtlasMap::BaseColor => self.base_color.as_ref(),
            AtlasMap::Normal => self.normal.as_ref(),
            AtlasMap::Height => self.height.as_ref(),
            AtlasMap::AmbientOcclusion => self.ambient_occlusion.as_ref(),
            AtlasMap::Roughness => self.roughness.as_ref(),
            AtlasMap::Metallic => self.metallic.as_ref(),
        }
    }

    pub fn get_mut(&mut self, map: AtlasMap) -> &mut Option<Handle<Image>> {
        match map {
            AtlasMap::BaseColor => &mut self.base_color,
            AtlasMap::Normal => &mut self.normal,
            AtlasMap::Height => &mut self.height,
            AtlasMap::AmbientOcclusion => &mut self.ambient_occlusion,
            AtlasMap::Roughness => &mut self.roughness,
            AtlasMap::Metallic => &mut self.metallic,
        }
    }
}

#[derive(Resource)]
pub struct Atlas {
    pub width: u32,
    pub height: u32,
    pub padding: u32,

    pub materials: Vec<AtlasMaterial>,

    /// Runtime GPU textures containing the composed atlas maps.
    pub images: AtlasImages,

    /// Changes that require mesh UVs to be regenerated.
    pub uv_revision: u64,

    /// Changes that require the composed atlas image to be rebuilt.
    pub image_revision: u64,

    pub bake_revision: u64,
}

impl Atlas {
    pub fn mark_uv_dirty(&mut self) {
        self.uv_revision = self.uv_revision.wrapping_add(1);
    }

    pub fn mark_image_dirty(&mut self) {
        self.image_revision = self.image_revision.wrapping_add(1);
    }

    /// Call this whenever a material's transform (x, y, width, height, rotation) is modified.
    pub fn mark_dirty(&mut self) {
        self.mark_uv_dirty();
        self.mark_image_dirty();
    }

    pub fn mark_bake_dirty(&mut self) {
        self.image_revision = self.image_revision.wrapping_add(1);
        self.bake_revision = self.bake_revision.wrapping_add(1);
    }

    fn find_free_rect(&mut self, width: u32, height: u32) -> (u32, u32) {
        const MIN_SIZE: u32 = 512;

        let padding = self.padding;

        self.width = self
            .width
            .max(MIN_SIZE)
            .max(width.saturating_add(padding * 2));

        self.height = self
            .height
            .max(MIN_SIZE)
            .max(height.saturating_add(padding * 2));

        loop {
            let mut y = padding;

            while y.saturating_add(height).saturating_add(padding) <= self.height {
                let mut x = padding;

                while x.saturating_add(width).saturating_add(padding) <= self.width {
                    let candidate = Rect {
                        x,
                        y,
                        width,
                        height,
                    };

                    let overlaps = self.materials.iter().any(|material| {
                        let (other_x, other_y, other_width, other_height) =
                            material.transform.positive_atlas_rect();

                        let other = Rect {
                            x: other_x,
                            y: other_y,
                            width: other_width,
                            height: other_height,
                        };

                        // Require exactly `padding` pixels between rectangles.
                        candidate.expanded(padding).overlaps(other)
                    });

                    if !overlaps {
                        return (x, y);
                    }

                    x = x.saturating_add(1);
                }

                y = y.saturating_add(1);
            }

            self.height = self.height.saturating_add(MIN_SIZE);

            if self.height == u32::MAX {
                panic!("atlas is too large to place a material");
            }
        }
    }

    pub fn next_index(&self) -> usize {
        self.materials
            .iter()
            .map(|material| material.index)
            .max()
            .map(|index| index + 1)
            .unwrap_or(0)
    }

    pub fn add_material(&mut self, source: MaterialSource) -> usize {
        let index = self.next_index();

        let textures = source
            .textures
            .into_iter()
            .map(|texture| MaterialTexture {
                id: texture.id,
                path: texture.path,
                group_id: texture.group_id,
                image: None,
                egui_texture: None,
            })
            .collect::<Vec<_>>();

        self.materials.push(AtlasMaterial {
            index,
            name: source.name,
            material_ref: source.material_ref,
            textures,
            transform: Transform::default(),
            mesh_assignments: Vec::new(),
        });

        index
    }

    pub fn add_material_and_load(
        &mut self,
        source: MaterialSource,
        images: &mut Assets<Image>,
    ) -> usize {
        let index = self.add_material(source);

        self.load_textures(images);

        index
    }

    pub fn add_placeholder(&mut self) -> usize {
        let index = self.next_index();
        let size = 128;
        let margin = 16;
        let mut x = margin;
        let mut y = margin;
        loop {
            let candidate = Rect {
                x,
                y,
                width: size,
                height: size,
            };

            let overlaps = self.materials.iter().any(|material| {
                let (x, y, width, height) = material.transform.positive_atlas_rect();

                candidate.overlaps_with_padding(
                    Rect {
                        x,
                        y,
                        width,
                        height,
                    },
                    self.padding,
                )
            });

            if !overlaps && x + size <= self.width && y + size <= self.height {
                break;
            }

            x += size + margin;
            if x + size > self.width {
                x = margin;
                y += size + margin;
            }

            if y + size > self.height {
                self.height += 512;
            }
        }

        self.uv_revision = self.uv_revision.wrapping_add(1);
        self.image_revision = self.image_revision.wrapping_add(1);

        index
    }

    pub fn remove(&mut self, index: usize) {
        let old_len = self.materials.len();

        self.materials.retain(|material| material.index != index);

        if self.materials.len() != old_len {
            self.uv_revision = self.uv_revision.wrapping_add(1);
            self.image_revision = self.image_revision.wrapping_add(1);
        }
    }

    pub fn load_textures(&mut self, images: &mut Assets<Image>) {
        let mut uv_changed = false;
        let mut image_changed = false;

        // First collect the textures that actually need decoding.
        let pending: Vec<(usize, usize, PathBuf)> = self
            .materials
            .iter()
            .enumerate()
            .flat_map(|(material_index, material)| {
                material
                    .textures
                    .iter()
                    .enumerate()
                    .filter(|(_, texture)| texture.image.is_none())
                    .map(move |(texture_index, texture)| {
                        (material_index, texture_index, texture.path.clone())
                    })
            })
            .collect();

        // Decode images in parallel.
        let decoded: Vec<(usize, usize, PathBuf, Result<Image, String>)> = pending
            .into_par_iter()
            .map(|(material_index, texture_index, path)| {
                let result = crate::image::import::load(&path).map_err(|error| error.to_string());

                (material_index, texture_index, path, result)
            })
            .collect();

        // Add decoded images to Bevy's Assets on the main thread.
        for (material_index, texture_index, path, result) in decoded {
            match result {
                Ok(image) => {
                    let size = (image.width(), image.height());

                    let material = &mut self.materials[material_index];
                    let texture = &mut material.textures[texture_index];

                    texture.image = Some(images.add(image));

                    image_changed = true;

                    // Store the dimensions for placement below.
                    // We handle placement after all textures are loaded.
                    if material.transform.width == 0 || material.transform.height == 0 {
                        let _ = size;
                    }
                }

                Err(error) => {
                    error!("Failed to load texture '{}': {error}", path.display());
                }
            }
        }

        // Determine placement after all images have been loaded.
        for material_index in 0..self.materials.len() {
            let mut preferred_size = None;

            {
                let material = &self.materials[material_index];

                for texture in &material.textures {
                    let Some(handle) = &texture.image else {
                        continue;
                    };

                    let Some(image) = images.get(handle) else {
                        continue;
                    };

                    if preferred_size.is_none() {
                        preferred_size = Some((image.width(), image.height()));
                    }
                }
            }

            let Some((width, height)) = preferred_size else {
                continue;
            };

            let needs_placement = {
                let material = &self.materials[material_index];

                material.transform.width == 0 || material.transform.height == 0
            };

            if !needs_placement {
                continue;
            }

            let (x, y) = self.find_free_rect(width, height);

            let material = &mut self.materials[material_index];

            material.transform = Transform {
                x: x as i32,
                y: y as i32,
                width,
                height,
                rotation: Rotation::Deg0,
            };

            uv_changed = true;
            image_changed = true;
        }

        if uv_changed {
            self.uv_revision = self.uv_revision.wrapping_add(1);
        }

        if image_changed {
            self.image_revision = self.image_revision.wrapping_add(1);
        }
    }

    pub fn add_material_from_existing(&mut self, source: &AtlasMaterial, name: String) -> usize {
        let index = self.next_index();

        self.materials.push(AtlasMaterial {
            index,
            name,
            material_ref: source.material_ref.clone(),
            textures: source
                .textures
                .iter()
                .map(|texture| MaterialTexture {
                    id: texture.id.clone(),
                    path: texture.path.clone(),
                    group_id: texture.group_id.clone(),
                    image: texture.image.clone(),
                    egui_texture: None,
                })
                .collect(),
            transform: Transform::default(),
            mesh_assignments: Vec::new(),
        });

        index
    }
}

impl Default for Atlas {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            padding: 16,

            materials: Vec::new(),
            images: AtlasImages::default(),

            uv_revision: 0,
            image_revision: 0,
            bake_revision: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialMeshAssignment {
    pub mesh_id: u64,
    pub object_index: usize,
    pub primitive_index: usize,
}

pub struct AtlasMaterial {
    pub index: usize,
    pub name: String,

    pub material_ref: Option<MeshMaterialRef>,

    pub textures: Vec<MaterialTexture>,

    pub transform: Transform,
    pub mesh_assignments: Vec<MaterialMeshAssignment>,
}

impl AtlasMaterial {
    pub fn has_mesh_assignment(
        &self,
        mesh_id: u64,
        object_index: usize,
        primitive_index: usize,
    ) -> bool {
        self.mesh_assignments.iter().any(|assignment| {
            assignment.mesh_id == mesh_id
                && assignment.object_index == object_index
                && assignment.primitive_index == primitive_index
        })
    }

    pub fn assign_mesh(&mut self, mesh_id: u64, object_index: usize, primitive_index: usize) {
        if self.has_mesh_assignment(mesh_id, object_index, primitive_index) {
            return;
        }

        self.mesh_assignments.push(MaterialMeshAssignment {
            mesh_id,
            object_index,
            primitive_index,
        });
    }

    pub fn unassign_mesh(&mut self, mesh_id: u64, object_index: usize, primitive_index: usize) {
        self.mesh_assignments.retain(|assignment| {
            assignment.mesh_id != mesh_id
                || assignment.object_index != object_index
                || assignment.primitive_index != primitive_index
        });
    }
}

#[derive(Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Rect {
    fn expanded(self, amount: u32) -> Self {
        Self {
            x: self.x.saturating_sub(amount),
            y: self.y.saturating_sub(amount),
            width: self.width.saturating_add(amount),
            height: self.height.saturating_add(amount),
        }
    }

    fn overlaps(self, other: Self) -> bool {
        self.x < other.x.saturating_add(other.width)
            && self.x.saturating_add(self.width) > other.x
            && self.y < other.y.saturating_add(other.height)
            && self.y.saturating_add(self.height) > other.y
    }

    fn overlaps_with_padding(self, other: Self, padding: u32) -> bool {
        self.x < other.x.saturating_add(other.width).saturating_add(padding)
            && self.x.saturating_add(self.width).saturating_add(padding) > other.x
            && self.y < other.y.saturating_add(other.height).saturating_add(padding)
            && self.y.saturating_add(self.height).saturating_add(padding) > other.y
    }
}

fn float_to_srgb8(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);

    let srgb = if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };

    (srgb * 255.0 + 0.5) as u8
}

fn read_source_pixel(image: &AtlasSourceImage, x: usize, y: usize) -> Option<[f32; 4]> {
    let pixel = y.checked_mul(image.width)?.checked_add(x)?;

    match image.format {
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm => {
            let offset = pixel.checked_mul(4)?;
            let data = image.data.get(offset..offset + 4)?;

            Some([
                data[0] as f32 / 255.0,
                data[1] as f32 / 255.0,
                data[2] as f32 / 255.0,
                data[3] as f32 / 255.0,
            ])
        }

        TextureFormat::Rgba32Float => {
            let offset = pixel.checked_mul(16)?;
            Some([
                read_f32(&image.data, offset)?,
                read_f32(&image.data, offset + 4)?,
                read_f32(&image.data, offset + 8)?,
                read_f32(&image.data, offset + 12)?,
            ])
        }

        TextureFormat::R32Float => {
            let offset = pixel.checked_mul(4)?;
            let value = read_f32(&image.data, offset)?;
            Some([value, value, value, 1.0])
        }

        TextureFormat::Rgba16Float => {
            let offset = pixel.checked_mul(8)?;
            Some([
                read_f16(&image.data, offset)?,
                read_f16(&image.data, offset + 2)?,
                read_f16(&image.data, offset + 4)?,
                read_f16(&image.data, offset + 6)?,
            ])
        }

        TextureFormat::R16Float => {
            let offset = pixel.checked_mul(2)?;
            let value = read_f16(&image.data, offset)?;
            Some([value, value, value, 1.0])
        }

        _ => None,
    }
}

fn source_coordinates(
    rotation: Rotation,
    dst_x: usize,
    dst_y: usize,
    dest_width: usize,
    dest_height: usize,
    source_width: usize,
    source_height: usize,
) -> (usize, usize) {
    if dest_width == 0 || dest_height == 0 || source_width == 0 || source_height == 0 {
        return (0, 0);
    }

    let x_source = dst_x.saturating_mul(source_width) / dest_width;
    let y_source = dst_y.saturating_mul(source_height) / dest_height;

    match rotation {
        Rotation::Deg0 => (x_source, y_source),

        Rotation::Deg90 => (
            dst_y.saturating_mul(source_width) / dest_height,
            source_height
                .saturating_sub(1)
                .saturating_sub(dst_x.saturating_mul(source_height) / dest_width),
        ),

        Rotation::Deg180 => (
            source_width.saturating_sub(1).saturating_sub(x_source),
            source_height.saturating_sub(1).saturating_sub(y_source),
        ),

        Rotation::Deg270 => (
            source_width
                .saturating_sub(1)
                .saturating_sub(dst_y.saturating_mul(source_width) / dest_height),
            dst_x.saturating_mul(source_height) / dest_width,
        ),
    }
}

fn atlas_destination_rect(transform: Transform) -> Option<(i64, i64, usize, usize)> {
    let (width, height) = match transform.rotation {
        Rotation::Deg0 | Rotation::Deg180 => (transform.width, transform.height),

        Rotation::Deg90 | Rotation::Deg270 => (transform.height, transform.width),
    };

    if width == 0 || height == 0 {
        return None;
    }

    Some((
        transform.x as i64,
        transform.y as i64,
        width as usize,
        height as usize,
    ))
}

fn make_atlas_image(width: usize, height: usize, data: Vec<u8>, format: TextureFormat) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        format,
        RenderAssetUsages::default(),
    );

    image.texture_descriptor.usage = bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
        | bevy::render::render_resource::TextureUsages::COPY_DST;

    image
}

pub(crate) fn rebuild_atlas_map(input: &AtlasBuildInput, map: AtlasMap) -> Image {
    let width = input.width;
    let height = input.height;

    let materials: Vec<PreparedAtlasMaterial<'_>> = input
        .materials
        .iter()
        .filter_map(|material| prepare_material(material, map))
        .collect();

    if matches!(map, AtlasMap::BaseColor) {
        rebuild_base_color_map(width, height, &materials)
    } else {
        rebuild_data_map(width, height, &materials)
    }
}

fn rebuild_data_map(width: usize, height: usize, materials: &[PreparedAtlasMaterial<'_>]) -> Image {
    let data_len = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(16))
        .expect("atlas dimensions are too large");

    let mut atlas_data = vec![0u8; data_len];

    const ROW_CHUNK: usize = 32;

    let row_bytes = width * 16;
    let chunk_bytes = row_bytes * ROW_CHUNK;

    atlas_data
        .par_chunks_mut(chunk_bytes)
        .enumerate()
        .for_each(|(chunk_index, chunk)| {
            let start_y = chunk_index * ROW_CHUNK;
            let end_y = (start_y + ROW_CHUNK).min(height);

            let active_materials: Vec<_> = materials
                .iter()
                .copied()
                .filter(|material| material_intersects_rows(material, start_y, end_y))
                .collect();

            for (local_y, row) in chunk.chunks_exact_mut(row_bytes).enumerate() {
                let atlas_y = start_y + local_y;
                let atlas_y = atlas_y as i64;

                for material in &active_materials {
                    if atlas_y < material.y || atlas_y >= material.y + material.height as i64 {
                        continue;
                    }

                    let dst_y = (atlas_y - material.y) as usize;

                    for dst_x in 0..material.width {
                        let atlas_x = material.x + dst_x as i64;

                        if atlas_x < 0 || atlas_x >= width as i64 {
                            continue;
                        }

                        let (src_x, src_y) = prepared_source_coordinates(material, dst_x, dst_y);

                        if src_x >= material.source_width || src_y >= material.source_height {
                            continue;
                        }

                        let Some(value) = read_source_pixel(material.image, src_x, src_y) else {
                            continue;
                        };

                        let dst_offset = atlas_x as usize * 16;

                        row[dst_offset..dst_offset + 4].copy_from_slice(&value[0].to_le_bytes());

                        row[dst_offset + 4..dst_offset + 8]
                            .copy_from_slice(&value[1].to_le_bytes());

                        row[dst_offset + 8..dst_offset + 12]
                            .copy_from_slice(&value[2].to_le_bytes());

                        row[dst_offset + 12..dst_offset + 16]
                            .copy_from_slice(&value[3].to_le_bytes());
                    }
                }
            }
        });

    make_atlas_image(width, height, atlas_data, TextureFormat::Rgba32Float)
}

fn rebuild_base_color_map(
    width: usize,
    height: usize,
    materials: &[PreparedAtlasMaterial<'_>],
) -> Image {
    let data_len = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .expect("atlas dimensions are too large");

    let mut atlas_data = vec![0u8; data_len];

    let row_bytes = width * 4;
    let chunk_bytes = row_bytes * ATLAS_ROW_CHUNK;

    atlas_data
        .par_chunks_mut(chunk_bytes)
        .enumerate()
        .for_each(|(chunk_index, chunk)| {
            let start_y = chunk_index * ATLAS_ROW_CHUNK;
            let end_y = (start_y + ATLAS_ROW_CHUNK).min(height);

            let active_materials: Vec<_> = materials
                .iter()
                .copied()
                .filter(|material| material_intersects_rows(material, start_y, end_y))
                .collect();

            for (local_y, row) in chunk.chunks_exact_mut(row_bytes).enumerate() {
                let atlas_y = (start_y + local_y) as i64;

                for material in &active_materials {
                    if atlas_y < material.y || atlas_y >= material.y + material.height as i64 {
                        continue;
                    }

                    let dst_y = (atlas_y - material.y) as usize;

                    for dst_x in 0..material.width {
                        let atlas_x = material.x + dst_x as i64;

                        if atlas_x < 0 || atlas_x >= width as i64 {
                            continue;
                        }

                        let (src_x, src_y) = prepared_source_coordinates(material, dst_x, dst_y);

                        if src_x >= material.source_width || src_y >= material.source_height {
                            continue;
                        }

                        let Some(value) = read_source_pixel(material.image, src_x, src_y) else {
                            continue;
                        };

                        let dst_offset = atlas_x as usize * 4;

                        row[dst_offset] = float_to_srgb8(value[0]);
                        row[dst_offset + 1] = float_to_srgb8(value[1]);
                        row[dst_offset + 2] = float_to_srgb8(value[2]);
                        row[dst_offset + 3] = (value[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                    }
                }
            }
        });

    make_atlas_image(width, height, atlas_data, TextureFormat::Rgba8UnormSrgb)
}

fn prepared_source_coordinates(
    material: &PreparedAtlasMaterial<'_>,
    dst_x: usize,
    dst_y: usize,
) -> (usize, usize) {
    source_coordinates(
        material.rotation,
        dst_x,
        dst_y,
        material.width,
        material.height,
        material.source_width,
        material.source_height,
    )
}

fn material_intersects_rows(
    material: &PreparedAtlasMaterial<'_>,
    start_y: usize,
    end_y: usize,
) -> bool {
    let mat_start_y = material.y;
    let mat_end_y = material.y + material.height as i64;

    let chunk_start_y = start_y as i64;
    let chunk_end_y = end_y as i64;

    mat_start_y < chunk_end_y && mat_end_y > chunk_start_y
}

pub(crate) fn create_atlas_build_input(
    atlas: &Atlas,
    texture_naming: &TextureNaming,
    images: &Assets<Image>,
) -> AtlasBuildInput {
    let materials = atlas
        .materials
        .iter()
        .map(|material| {
            let mut maps = BTreeMap::new();
            let mut channel_sources = Vec::new();

            for texture in &material.textures {
                let Some(group_id) = texture.group_id.as_deref() else {
                    continue;
                };

                let Some(group) = texture_naming
                    .groups
                    .iter()
                    .find(|group| group.id == group_id)
                else {
                    continue;
                };

                add_group_rendering_bindings(
                    &mut maps,
                    &mut channel_sources,
                    material,
                    texture,
                    group,
                    images,
                );
            }

            AtlasMaterialBuildInput {
                transform: material.transform,
                maps,
                channel_sources,
            }
        })
        .collect();

    AtlasBuildInput {
        width: atlas.width as usize,
        height: atlas.height as usize,
        materials,
    }
}

pub(crate) fn bake_channel_maps(input: &mut AtlasBuildInput) {
    for material in &mut input.materials {
        let channel_sources = std::mem::take(&mut material.channel_sources);

        for (map, source, channel) in channel_sources {
            let Some(baked) = bake_channel_source(&source, channel) else {
                continue;
            };

            material.maps.insert(map, baked);
        }
    }
}

pub(crate) fn build_atlas(mut input: AtlasBuildInput, generation: u64) -> AtlasBuildResult {
    bake_channel_maps(&mut input);

    let maps = [
        AtlasMap::BaseColor,
        AtlasMap::Normal,
        AtlasMap::Height,
        AtlasMap::AmbientOcclusion,
        AtlasMap::Roughness,
        AtlasMap::Metallic,
    ];

    let maps = maps
        .into_iter()
        .filter_map(|map| {
            if !atlas_input_has_map(&input, map) {
                return None;
            }

            Some((map, rebuild_atlas_map(&input, map)))
        })
        .collect();

    AtlasBuildResult { generation, maps }
}

fn atlas_input_has_map(input: &AtlasBuildInput, map: AtlasMap) -> bool {
    input
        .materials
        .iter()
        .any(|material| material.maps.contains_key(&map))
}
