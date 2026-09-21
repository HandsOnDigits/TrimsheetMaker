#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Transform {
    pub x: i32,
    pub y: i32,

    // Original/source dimensions. NEVER change these because of rotation.
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

    pub fn signed_atlas_rect(&self) -> (i64, i64, u32, u32) {
        let (center_x, center_y) = self.center();

        let width = self.rotated_width();
        let height = self.rotated_height();

        let x = center_x - width as i64 / 2;
        let y = center_y - height as i64 / 2;

        (x, y, width, height)
    }

    /// Returns a non-negative rectangle for atlas packing operations.
    pub fn positive_atlas_rect(&self) -> (u32, u32, u32, u32) {
        let (x, y, width, height) = self.signed_atlas_rect();

        (x.max(0) as u32, y.max(0) as u32, width, height)
    }

    pub fn atlas_rect(&self) -> (i64, i64, u32, u32) {
        self.signed_atlas_rect()
    }

    pub fn rotate_cw(&mut self) {
        self.rotation = self.rotation.next();
    }

    pub fn rotate_ccw(&mut self) {
        self.rotation = self.rotation.previous();
    }
}
