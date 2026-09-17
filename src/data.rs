use bevy::prelude::*;
use bevy_egui::egui;

use std::collections::BTreeMap;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureGroup {
    pub id: String,
    pub name: String,
    pub suffixes: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureNaming {
    pub groups: Vec<TextureGroup>,
}

impl TextureNaming {
    pub fn reclassify_atlas(&self, atlas: &mut Atlas) {
        for material in &mut atlas.materials {
            for texture in &mut material.textures {
                texture.id = self
                    .group_for_filename(
                        texture
                            .path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or_default(),
                    )
                    .unwrap_or_default();
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
        self.groups
            .iter()
            .flat_map(|group| group.suffixes.iter().map(move |suffix| (group, suffix)))
            .filter_map(|(group, suffix)| {
                filename
                    .to_ascii_lowercase()
                    .ends_with(&suffix.to_ascii_lowercase())
                    .then_some((suffix.len(), &group.id))
            })
            .max_by_key(|(length, _)| *length)
            .map(|(_, id)| id.clone())
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

#[derive(Resource, Default)]
pub struct Atlas {
    pub width: u32,
    pub height: u32,
    pub materials: Vec<AtlasMaterial>,
}

impl Atlas {
    fn find_free_rect(&mut self, width: u32, height: u32) -> (u32, u32) {
        const MARGIN: u32 = 16;
        const MIN_SIZE: u32 = 512;

        // Make sure the atlas can contain the requested rectangle at all.
        let required_width = width.saturating_add(MARGIN * 2);
        let required_height = height.saturating_add(MARGIN * 2);

        if self.width < required_width {
            self.width = self.width.max(MIN_SIZE).max(required_width);
        }

        if self.height < required_height {
            self.height = self.height.max(MIN_SIZE).max(required_height);
        }

        loop {
            let mut y = MARGIN;

            while y.saturating_add(height) <= self.height {
                let mut x = MARGIN;

                while x.saturating_add(width) <= self.width {
                    let candidate = Rect {
                        x,
                        y,
                        width,
                        height,
                    };

                    let overlaps = self.materials.iter().any(|material| {
                        if material.width == 0 || material.height == 0 {
                            return false;
                        }

                        candidate.overlaps(Rect {
                            x: material.x,
                            y: material.y,
                            width: material.width,
                            height: material.height,
                        })
                    });

                    if !overlaps {
                        return (x, y);
                    }

                    x = x.saturating_add(width + MARGIN);
                }

                y = y.saturating_add(height + MARGIN);
            }

            // No room: grow vertically and try again.
            self.height = self.height.saturating_add(MIN_SIZE);

            // Prevent an impossible/infinite search if u32 is exhausted.
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
            x: 0,
            y: 0,
            width: 0,
            height: 0,
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
                candidate.overlaps(Rect {
                    x: material.x,
                    y: material.y,
                    width: material.width,
                    height: material.height,
                })
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
            x,
            y,
            width: size,
            height: size,
        });
        index
    }

    pub fn remove(&mut self, index: usize) {
        self.materials.retain(|material| material.index != index);
    }

    pub fn load_textures(&mut self, images: &mut Assets<Image>) {
        for material_index in 0..self.materials.len() {
            let mut material_size = None;

            {
                let material = &mut self.materials[material_index];

                for texture in &mut material.textures {
                    if texture.image.is_some() {
                        continue;
                    }

                    match crate::assets::exr::load_exr(&texture.path) {
                        Ok(image) => {
                            let width = image.width();
                            let height = image.height();

                            if material_size.is_none() {
                                material_size = Some((width, height));
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

            if let Some((width, height)) = material_size {
                let needs_placement = {
                    let material = &self.materials[material_index];
                    material.width == 0 || material.height == 0
                };

                if needs_placement {
                    let (x, y) = self.find_free_rect(width, height);

                    let material = &mut self.materials[material_index];

                    material.x = x;
                    material.y = y;
                    material.width = width;
                    material.height = height;
                }
            }
        }
    }
}

pub struct AtlasMaterial {
    pub index: usize,
    pub name: String,

    pub textures: Vec<MaterialTexture>,

    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Rect {
    fn overlaps(self, other: Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}
