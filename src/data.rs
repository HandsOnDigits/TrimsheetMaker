use bevy::prelude::*;
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
        });

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
        });

        index
    }

    pub fn remove(&mut self, index: usize) {
        self.materials.retain(|material| material.index != index);
    }

    pub fn load_textures(&mut self, images: &mut Assets<Image>, preferred_group: Option<&str>) {
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
        }
    }
}

pub struct AtlasMaterial {
    pub index: usize,
    pub name: String,

    pub textures: Vec<MaterialTexture>,

    pub transform: Transform,
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
