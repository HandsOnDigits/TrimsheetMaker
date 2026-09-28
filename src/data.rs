use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use bevy_egui::egui;

use std::collections::BTreeMap;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::transform::{Rotation, Transform};

#[derive(Clone)]
pub struct MaterialTexture {
    pub id: String,

    // Original source file. Never discard this.
    pub path: PathBuf,

    // Runtime representation, if loaded.
    pub image: Option<Handle<Image>>,
    pub egui_texture: Option<egui::TextureId>,
}

pub struct MaterialSource {
    pub name: String,
    pub textures: Vec<MaterialTexture>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TextureGroup {
    pub id: String,
    pub name: String,
    pub suffixes: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TextureNaming {
    pub groups: Vec<TextureGroup>,
}

impl TextureNaming {
    pub fn reclassify_atlas(&self, atlas: &mut Atlas) {
        for material in &mut atlas.materials {
            for texture in &mut material.textures {
                if let Some(group) = self.group_for_filename(
                    texture
                        .path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default(),
                ) {
                    texture.id = group;
                }
            }
        }
    }

    pub fn reclassify_materials(&self, materials: &mut BTreeMap<String, MaterialSource>) {
        for material in materials.values_mut() {
            for texture in &mut material.textures {
                if let Some(group) = self.group_for_filename(
                    texture
                        .path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default(),
                ) {
                    texture.id = group;
                }
            }
        }
    }

    pub fn group_for_filename(&self, filename: &str) -> Option<String> {
        let stem = std::path::Path::new(filename)
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
        Self { groups: Vec::new() }
    }
}

#[derive(Resource)]
pub struct Atlas {
    pub width: u32,
    pub height: u32,

    pub padding: u32,

    pub materials: Vec<AtlasMaterial>,

    /// Runtime GPU texture containing the composed atlas.
    pub image: Option<Handle<Image>>,

    /// Changes that require mesh UVs to be regenerated.
    pub uv_revision: u64,

    /// Changes that require the composed atlas image to be rebuilt.
    pub image_revision: u64,
}

impl Atlas {
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

    pub fn selected_mut(&mut self, index: Option<usize>) -> Option<&mut AtlasMaterial> {
        index.and_then(|index| {
            self.materials
                .iter_mut()
                .find(|material| material.index == index)
        })
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
                image: None,
                egui_texture: None,
            })
            .collect();

        self.materials.push(AtlasMaterial {
            index,
            name: source.name,
            textures,
            transform: Transform::default(),
            mesh_assignments: Vec::new(),
        });

        self.uv_revision = self.uv_revision.wrapping_add(1);

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

        self.materials.push(AtlasMaterial {
            index,
            name: format!("Material {index}"),
            textures: vec![],
            transform: Transform {
                x: x as i32,
                y: y as i32,
                width: size,
                height: size,
                ..Default::default()
            },
            mesh_assignments: Vec::new(),
        });

        self.uv_revision = self.uv_revision.wrapping_add(1);

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

    pub fn load_textures(&mut self, images: &mut Assets<Image>, preferred_group: Option<&str>) {
        let mut uv_changed = false;
        let mut image_changed = false;

        for material_index in 0..self.materials.len() {
            let mut first_size = None;
            let mut preferred_size = None;

            {
                let material = &mut self.materials[material_index];

                for texture in &mut material.textures {
                    if texture.image.is_some() {
                        continue;
                    }

                    match crate::image::import::exr::load_exr(&texture.path) {
                        Ok(image) => {
                            let size = (image.width(), image.height());

                            if first_size.is_none() {
                                first_size = Some(size);
                            }

                            if preferred_group
                                .is_some_and(|group| texture.id.eq_ignore_ascii_case(group))
                            {
                                preferred_size = Some(size);
                            }

                            texture.image = Some(images.add(image));

                            image_changed = true;
                        }

                        Err(error) => {
                            error!(
                                "Failed to load texture '{}': {error}",
                                texture.path.display()
                            );
                        }
                    }
                }
            }

            let Some((width, height)) = preferred_size.or(first_size) else {
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
}

impl Default for Atlas {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            padding: 16,
            materials: Vec::new(),
            image: None,
            uv_revision: 0,
            image_revision: 0,
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

pub fn rebuild_atlas_image(
    atlas: &mut Atlas,
    images: &mut Assets<Image>,
    preferred_group: Option<&str>,
) {
    println!(
        "REBUILD: {}x{}, materials={}",
        atlas.width,
        atlas.height,
        atlas.materials.len()
    );

    let width = atlas.width as usize;
    let height = atlas.height as usize;

    if width == 0 || height == 0 {
        atlas.image = None;
        return;
    }

    let mut atlas_data = vec![0u8; width * height * 4];

    for material in &atlas.materials {
        // Prefer the classified texture ID, but fall back to the filename
        // while debugging/import classification.
        let texture = preferred_group
            .and_then(|group| {
                material
                    .textures
                    .iter()
                    .find(|texture| texture.id.eq_ignore_ascii_case(group))
            })
            .or_else(|| {
                material.textures.iter().find(|texture| {
                    texture
                        .path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .is_some_and(|stem| {
                            let stem = stem.to_ascii_lowercase();

                            stem.ends_with("basecolor") || stem.ends_with("base_color")
                        })
                })
            });

        let Some(texture) = texture else {
            println!("NO BASE COLOR FOUND: {}", material.name);
            continue;
        };

        println!(
            "USING BASE COLOR: {} -> {}",
            material.name,
            texture.path.display()
        );

        let Some(image_handle) = texture.image.as_ref() else {
            println!("TEXTURE NOT LOADED: {}", texture.path.display());
            continue;
        };

        let Some(image) = images.get(image_handle) else {
            println!("IMAGE HANDLE INVALID: {}", texture.path.display());
            continue;
        };

        let Some(source_data) = image.data.as_ref() else {
            println!("IMAGE HAS NO DATA: {}", texture.path.display());
            continue;
        };

        let source_width = image.width() as usize;
        let source_height = image.height() as usize;

        println!(
            "SOURCE: {}x{} format={:?}",
            source_width, source_height, image.texture_descriptor.format
        );

        if source_width == 0 || source_height == 0 {
            continue;
        }

        let transform = material.transform;

        let region_width = transform.width as usize;
        let region_height = transform.height as usize;

        if region_width == 0 || region_height == 0 {
            println!("INVALID REGION: {}", material.name);
            continue;
        }

        for dst_y in 0..region_height {
            for dst_x in 0..region_width {
                let (src_x, src_y) = match transform.rotation {
                    Rotation::Deg0 => (
                        dst_x * source_width / region_width,
                        dst_y * source_height / region_height,
                    ),

                    Rotation::Deg90 => (
                        dst_y * source_width / region_height,
                        source_height
                            .saturating_sub(1)
                            .saturating_sub(dst_x * source_height / region_width),
                    ),

                    Rotation::Deg180 => (
                        source_width
                            .saturating_sub(1)
                            .saturating_sub(dst_x * source_width / region_width),
                        source_height
                            .saturating_sub(1)
                            .saturating_sub(dst_y * source_height / region_height),
                    ),

                    Rotation::Deg270 => (
                        source_width
                            .saturating_sub(1)
                            .saturating_sub(dst_y * source_width / region_height),
                        dst_x * source_height / region_width,
                    ),
                };

                if src_x >= source_width || src_y >= source_height {
                    continue;
                }

                let dst_x_atlas = transform.x + dst_x as i32;
                let dst_y_atlas = transform.y + dst_y as i32;

                if dst_x_atlas < 0
                    || dst_y_atlas < 0
                    || dst_x_atlas >= width as i32
                    || dst_y_atlas >= height as i32
                {
                    continue;
                }

                let dst_index = (dst_y_atlas as usize * width + dst_x_atlas as usize) * 4;

                match image.texture_descriptor.format {
                    TextureFormat::Rgba32Float => {
                        let src_index = (src_y * source_width + src_x) * 16;

                        let r = f32::from_le_bytes([
                            source_data[src_index],
                            source_data[src_index + 1],
                            source_data[src_index + 2],
                            source_data[src_index + 3],
                        ]);

                        let g = f32::from_le_bytes([
                            source_data[src_index + 4],
                            source_data[src_index + 5],
                            source_data[src_index + 6],
                            source_data[src_index + 7],
                        ]);

                        let b = f32::from_le_bytes([
                            source_data[src_index + 8],
                            source_data[src_index + 9],
                            source_data[src_index + 10],
                            source_data[src_index + 11],
                        ]);

                        let a = f32::from_le_bytes([
                            source_data[src_index + 12],
                            source_data[src_index + 13],
                            source_data[src_index + 14],
                            source_data[src_index + 15],
                        ]);

                        if dst_x == 0 && dst_y == 0 {
                            println!("EXR SAMPLE: r={:.4}, g={:.4}, b={:.4}, a={:.4}", r, g, b, a);
                        }

                        atlas_data[dst_index] = float_to_srgb8(r);
                        atlas_data[dst_index + 1] = float_to_srgb8(g);
                        atlas_data[dst_index + 2] = float_to_srgb8(b);
                        atlas_data[dst_index + 3] = (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                    }

                    TextureFormat::R32Float => {
                        let src_index = (src_y * source_width + src_x) * 4;

                        let value = f32::from_le_bytes([
                            source_data[src_index],
                            source_data[src_index + 1],
                            source_data[src_index + 2],
                            source_data[src_index + 3],
                        ]);

                        let value = float_to_srgb8(value);

                        atlas_data[dst_index] = value;
                        atlas_data[dst_index + 1] = value;
                        atlas_data[dst_index + 2] = value;
                        atlas_data[dst_index + 3] = 255;
                    }

                    format => {
                        println!("UNSUPPORTED SOURCE FORMAT: {:?}", format);
                        break;
                    }
                }
            }
        }
    }

    let image = Image::new(
        Extent3d {
            width: atlas.width,
            height: atlas.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    if let Some(handle) = atlas.image.clone() {
        if let Some(mut existing) = images.get_mut(&handle) {
            *existing = image;
        } else {
            atlas.image = Some(images.add(image));
        }
    } else {
        atlas.image = Some(images.add(image));
    }
}
