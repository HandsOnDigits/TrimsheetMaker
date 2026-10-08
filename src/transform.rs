use serde::{Deserialize, Serialize};

use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Rotation {
    pub fn next(self) -> Self {
        match self {
            Self::Deg0 => Self::Deg90,
            Self::Deg90 => Self::Deg180,
            Self::Deg180 => Self::Deg270,
            Self::Deg270 => Self::Deg0,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Deg0 => Self::Deg270,
            Self::Deg90 => Self::Deg0,
            Self::Deg180 => Self::Deg90,
            Self::Deg270 => Self::Deg180,
        }
    }

    pub fn degrees(self) -> u32 {
        match self {
            Self::Deg0 => 0,
            Self::Deg90 => 90,
            Self::Deg180 => 180,
            Self::Deg270 => 270,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transform {
    pub x: i32,
    pub y: i32,

    pub width: u32,
    pub height: u32,

    pub rotation: Rotation,
}

impl Transform {
    pub fn center(&self) -> (i64, i64) {
        (
            self.x as i64 + self.width as i64 / 2,
            self.y as i64 + self.height as i64 / 2,
        )
    }

    pub fn rotated_width(&self) -> u32 {
        match self.rotation {
            Rotation::Deg0 | Rotation::Deg180 => self.width,
            Rotation::Deg90 | Rotation::Deg270 => self.height,
        }
    }

    pub fn rotated_height(&self) -> u32 {
        match self.rotation {
            Rotation::Deg0 | Rotation::Deg180 => self.height,
            Rotation::Deg90 | Rotation::Deg270 => self.width,
        }
    }

    pub fn atlas_rect(&self) -> (i64, i64, u32, u32) {
        let (center_x, center_y) = self.center();

        let width = self.rotated_width();
        let height = self.rotated_height();

        let x = center_x - width as i64 / 2;
        let y = center_y - height as i64 / 2;

        (x, y, width, height)
    }

    pub fn positive_atlas_rect(&self) -> (u32, u32, u32, u32) {
        let (x, y, width, height) = self.atlas_rect();

        (x.max(0) as u32, y.max(0) as u32, width, height)
    }

    pub fn rotate_cw(&mut self) {
        self.rotation = self.rotation.next();
    }

    pub fn rotate_ccw(&mut self) {
        self.rotation = self.rotation.previous();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvTransform {
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for UvTransform {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl UvTransform {
    pub fn uv_center(uvs: &[[f32; 2]]) -> Vec2 {
        if uvs.is_empty() {
            return Vec2::splat(0.5);
        }

        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);

        for &[x, y] in uvs {
            let uv = Vec2::new(x, y);
            min = min.min(uv);
            max = max.max(uv);
        }

        (min + max) * 0.5
    }

    pub fn apply(&self, uv: Vec2, pivot: Vec2) -> Vec2 {
        let mut result = uv - pivot;

        result *= self.scale;

        let rotation = Mat2::from_angle(self.rotation);
        result = rotation * result;

        result + pivot + self.translation
    }
}
