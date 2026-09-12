use bevy::prelude::*;
use bevy_egui::{EguiPrimaryContextPass, egui};

mod panels;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorUi>()
            .init_resource::<Atlas>()
            .add_systems(Startup, setup)
            .add_systems(EguiPrimaryContextPass, panels::editor_ui);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource)]
pub struct EditorUi {
    pub selected_material: Option<usize>,
    pub atlas_zoom: f32,
    pub atlas_pan: egui::Vec2,

    pub show_materials: bool,
    pub show_inspector: bool,
    pub show_status: bool,
}

impl Default for EditorUi {
    fn default() -> Self {
        Self {
            selected_material: Some(0),
            atlas_zoom: 1.0,
            atlas_pan: egui::Vec2::ZERO,
            show_materials: true,
            show_inspector: true,
            show_status: true,
        }
    }
}

#[derive(Resource)]
pub struct Atlas {
    pub width: u32,
    pub height: u32,
    pub materials: Vec<AtlasMaterial>,
}

impl Default for Atlas {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
            materials: vec![
                AtlasMaterial {
                    index: 0,
                    name: "Brick".into(),
                    textures: vec![
                        MaterialTexture {
                            name: "Albedo".into(),
                            path: "brick_albedo.png".into(),
                        },
                        MaterialTexture {
                            name: "Normal".into(),
                            path: "brick_normal.png".into(),
                        },
                        MaterialTexture {
                            name: "Roughness".into(),
                            path: "brick_roughness.png".into(),
                        },
                    ],
                    x: 32,
                    y: 32,
                    width: 256,
                    height: 256,
                },
                AtlasMaterial {
                    index: 1,
                    name: "Metal".into(),
                    textures: vec![
                        MaterialTexture {
                            name: "Albedo".into(),
                            path: "metal_albedo.png".into(),
                        },
                        MaterialTexture {
                            name: "Normal".into(),
                            path: "metal_normal.png".into(),
                        },
                        MaterialTexture {
                            name: "Metallic".into(),
                            path: "metal_metallic.png".into(),
                        },
                        MaterialTexture {
                            name: "Roughness".into(),
                            path: "metal_roughness.png".into(),
                        },
                    ],
                    x: 320,
                    y: 32,
                    width: 128,
                    height: 128,
                },
                AtlasMaterial {
                    index: 2,
                    name: "Wood".into(),
                    textures: vec![
                        MaterialTexture {
                            name: "Albedo".into(),
                            path: "wood_albedo.png".into(),
                        },
                        MaterialTexture {
                            name: "Normal".into(),
                            path: "wood_normal.png".into(),
                        },
                        MaterialTexture {
                            name: "Roughness".into(),
                            path: "wood_roughness.png".into(),
                        },
                    ],
                    x: 480,
                    y: 32,
                    width: 256,
                    height: 128,
                },
                AtlasMaterial {
                    index: 3,
                    name: "Concrete".into(),
                    textures: vec![
                        MaterialTexture {
                            name: "Albedo".into(),
                            path: "concrete_albedo.png".into(),
                        },
                        MaterialTexture {
                            name: "Normal".into(),
                            path: "concrete_normal.png".into(),
                        },
                        MaterialTexture {
                            name: "Roughness".into(),
                            path: "concrete_roughness.png".into(),
                        },
                    ],
                    x: 32,
                    y: 320,
                    width: 192,
                    height: 192,
                },
            ],
        }
    }
}

impl Atlas {
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
}

#[derive(Clone)]
pub struct AtlasMaterial {
    pub index: usize,
    pub name: String,

    pub textures: Vec<MaterialTexture>,

    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone)]
pub struct MaterialTexture {
    pub name: String,
    pub path: String,
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
