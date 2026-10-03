use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    Red,
    Green,
    Blue,
    Alpha,
}

impl Channel {
    pub const ALL: [Self; 4] = [Self::Red, Self::Green, Self::Blue, Self::Alpha];

    pub fn name(self) -> &'static str {
        match self {
            Self::Red => "R",
            Self::Green => "G",
            Self::Blue => "B",
            Self::Alpha => "A",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelSource {
    pub channel: Channel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderingUsage {
    Sample,
    Channel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialInput {
    BaseColor,
    Normal,
    Height,
    AmbientOcclusion,
    Roughness,
    Metallic,
}

impl MaterialInput {
    pub const ALL: [Self; 6] = [
        Self::BaseColor,
        Self::Normal,
        Self::Height,
        Self::AmbientOcclusion,
        Self::Roughness,
        Self::Metallic,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::BaseColor => "Base Color",
            Self::Normal => "Normal",
            Self::Height => "Height",
            Self::AmbientOcclusion => "Ambient Occlusion",
            Self::Roughness => "Roughness",
            Self::Metallic => "Metallic",
        }
    }

    pub fn usage(self) -> RenderingUsage {
        match self {
            Self::BaseColor | Self::Normal => RenderingUsage::Sample,
            Self::Height | Self::AmbientOcclusion | Self::Roughness | Self::Metallic => {
                RenderingUsage::Channel
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderingMaterialBindings {
    pub base_color: bool,
    pub normal: bool,

    pub height: Option<ChannelSource>,
    pub ambient_occlusion: Option<ChannelSource>,
    pub roughness: Option<ChannelSource>,
    pub metallic: Option<ChannelSource>,
}
