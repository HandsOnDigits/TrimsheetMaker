use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

use crate::mesh::import::MeshMaterialRef;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderingMaterialBindings {
    pub base_color: Option<TextureBinding>,
    pub normal: Option<TextureBinding>,

    pub height: Option<ChannelBinding>,
    pub ambient_occlusion: Option<ChannelBinding>,
    pub roughness: Option<ChannelBinding>,
    pub metallic: Option<ChannelBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureBinding {
    pub texture_group: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelBinding {
    pub texture_group: String,
    pub channel: TextureChannel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureChannel {
    Red,
    Green,
    Blue,
    Alpha,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PipelineNodeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderingNodeKind {
    Texture,
    SplitChannels,
    MaterialOutput,
}

impl RenderingNodeKind {
    pub fn input_sockets(self) -> &'static [RenderingInputSocket] {
        match self {
            Self::Texture => &[],

            Self::SplitChannels => &[RenderingInputSocket::Texture],

            Self::MaterialOutput => &[
                RenderingInputSocket::BaseColor,
                RenderingInputSocket::Normal,
                RenderingInputSocket::Height,
                RenderingInputSocket::AmbientOcclusion,
                RenderingInputSocket::Roughness,
                RenderingInputSocket::Metallic,
            ],
        }
    }

    pub fn output_sockets(self) -> &'static [RenderingOutputSocket] {
        match self {
            Self::Texture => &[RenderingOutputSocket::Texture],

            Self::SplitChannels => &[
                RenderingOutputSocket::Red,
                RenderingOutputSocket::Green,
                RenderingOutputSocket::Blue,
                RenderingOutputSocket::Alpha,
            ],

            Self::MaterialOutput => &[],
        }
    }

    pub fn accepts_input(self, socket: RenderingInputSocket) -> bool {
        self.input_sockets().contains(&socket)
    }

    pub fn provides_output(self, socket: RenderingOutputSocket) -> bool {
        self.output_sockets().contains(&socket)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderingInputSocket {
    Texture,

    BaseColor,
    Normal,
    Height,
    AmbientOcclusion,
    Roughness,
    Metallic,
}

impl RenderingInputSocket {
    pub fn material_outputs() -> &'static [Self] {
        &[
            Self::BaseColor,
            Self::Normal,
            Self::Height,
            Self::AmbientOcclusion,
            Self::Roughness,
            Self::Metallic,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderingValueType {
    Texture,
    Channel,
}

impl RenderingInputSocket {
    pub fn value_type(self) -> RenderingValueType {
        match self {
            Self::Texture | Self::BaseColor | Self::Normal => RenderingValueType::Texture,

            Self::Height | Self::AmbientOcclusion | Self::Roughness | Self::Metallic => {
                RenderingValueType::Channel
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderingOutputSocket {
    Texture,

    Red,
    Green,
    Blue,
    Alpha,
}

impl RenderingOutputSocket {
    pub fn value_type(self) -> RenderingValueType {
        match self {
            Self::Texture => RenderingValueType::Texture,

            Self::Red | Self::Green | Self::Blue | Self::Alpha => RenderingValueType::Channel,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineInput {
    pub node_id: PipelineNodeId,
    pub socket: RenderingInputSocket,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineOutput {
    pub node_id: PipelineNodeId,
    pub socket: RenderingOutputSocket,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderingConnection {
    pub from: PipelineOutput,
    pub to: PipelineInput,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineNode {
    pub id: PipelineNodeId,
    pub kind: RenderingNodeKind,

    /// Used by Texture nodes.
    pub texture_group: Option<String>,
}

#[derive(Clone, Debug)]
enum ResolvedValue {
    Texture(String),
    Channel(ChannelBinding),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RenderingPipeline {
    pub nodes: Vec<PipelineNode>,
    pub connections: Vec<RenderingConnection>,

    #[serde(default)]
    next_node_id: u64,
}

impl RenderingPipeline {
    // -------------------------------------------------------------------------
    // Nodes
    // -------------------------------------------------------------------------

    pub fn add_node(&mut self, kind: RenderingNodeKind) -> PipelineNodeId {
        if kind == RenderingNodeKind::MaterialOutput {
            if let Some(existing) = self
                .nodes
                .iter()
                .find(|node| node.kind == RenderingNodeKind::MaterialOutput)
            {
                return existing.id;
            }
        }

        let id = PipelineNodeId(self.next_node_id);
        self.next_node_id += 1;

        self.nodes.push(PipelineNode {
            id,
            kind,
            texture_group: None,
        });

        id
    }

    pub fn remove_node(&mut self, id: PipelineNodeId) -> bool {
        let old_len = self.nodes.len();

        self.nodes.retain(|node| node.id != id);

        self.connections
            .retain(|connection| connection.from.node_id != id && connection.to.node_id != id);

        self.nodes.len() != old_len
    }

    pub fn node(&self, id: PipelineNodeId) -> Option<&PipelineNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn node_mut(&mut self, id: PipelineNodeId) -> Option<&mut PipelineNode> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }

    pub fn material_output(&self) -> Option<&PipelineNode> {
        self.nodes
            .iter()
            .find(|node| node.kind == RenderingNodeKind::MaterialOutput)
    }

    // -------------------------------------------------------------------------
    // Connections
    // -------------------------------------------------------------------------

    pub fn connect(
        &mut self,
        from: PipelineOutput,
        to: PipelineInput,
    ) -> Result<(), RenderingPipelineError> {
        let source = self
            .node(from.node_id)
            .ok_or(RenderingPipelineError::MissingNode {
                node_id: from.node_id,
            })?;

        let target = self
            .node(to.node_id)
            .ok_or(RenderingPipelineError::MissingNode {
                node_id: to.node_id,
            })?;

        if !source.kind.provides_output(from.socket) {
            return Err(RenderingPipelineError::InvalidOutput {
                node_id: from.node_id,
                socket: from.socket,
            });
        }

        if !target.kind.accepts_input(to.socket) {
            return Err(RenderingPipelineError::InvalidInput {
                node_id: to.node_id,
                socket: to.socket,
            });
        }

        if self.would_create_cycle(from.node_id, to.node_id) {
            return Err(RenderingPipelineError::Cycle {
                node_id: to.node_id,
            });
        }

        self.disconnect(to);

        self.connections.push(RenderingConnection { from, to });

        Ok(())
    }

    pub fn disconnect(&mut self, input: PipelineInput) -> bool {
        let old_len = self.connections.len();

        self.connections.retain(|connection| connection.to != input);

        self.connections.len() != old_len
    }

    pub fn disconnect_output(&mut self, output: PipelineOutput) -> usize {
        let old_len = self.connections.len();

        self.connections
            .retain(|connection| connection.from != output);

        old_len - self.connections.len()
    }

    pub fn connection_to(&self, input: PipelineInput) -> Option<&RenderingConnection> {
        self.connections
            .iter()
            .find(|connection| connection.to == input)
    }

    fn would_create_cycle(&self, from: PipelineNodeId, to: PipelineNodeId) -> bool {
        if from == to {
            return true;
        }

        let mut stack = vec![to];
        let mut visited = Vec::new();

        while let Some(node_id) = stack.pop() {
            if node_id == from {
                return true;
            }

            if visited.contains(&node_id) {
                continue;
            }

            visited.push(node_id);

            for connection in &self.connections {
                if connection.from.node_id == node_id {
                    stack.push(connection.to.node_id);
                }
            }
        }

        false
    }

    // -------------------------------------------------------------------------
    // Compilation
    // -------------------------------------------------------------------------

    pub fn compile(&self) -> Result<RenderingMaterialBindings, RenderingPipelineError> {
        let output = self
            .material_output()
            .ok_or(RenderingPipelineError::MissingMaterialOutput)?;

        let mut bindings = RenderingMaterialBindings::default();

        for socket in RenderingInputSocket::material_outputs() {
            let input = PipelineInput {
                node_id: output.id,
                socket: *socket,
            };

            let Some(connection) = self.connection_to(input) else {
                continue;
            };

            let value = self.resolve_output(connection.from)?;

            match (socket, value) {
                (RenderingInputSocket::BaseColor, ResolvedValue::Texture(texture_group)) => {
                    bindings.base_color = Some(TextureBinding { texture_group });
                }

                (RenderingInputSocket::Normal, ResolvedValue::Texture(texture_group)) => {
                    bindings.normal = Some(TextureBinding { texture_group });
                }

                (RenderingInputSocket::Height, ResolvedValue::Channel(channel)) => {
                    bindings.height = Some(channel);
                }

                (RenderingInputSocket::AmbientOcclusion, ResolvedValue::Channel(channel)) => {
                    bindings.ambient_occlusion = Some(channel);
                }

                (RenderingInputSocket::Roughness, ResolvedValue::Channel(channel)) => {
                    bindings.roughness = Some(channel);
                }

                (RenderingInputSocket::Metallic, ResolvedValue::Channel(channel)) => {
                    bindings.metallic = Some(channel);
                }

                (socket, value) => {
                    return Err(RenderingPipelineError::InvalidConnection {
                        message: format!("Invalid value {:?} connected to {:?}", value, socket),
                    });
                }
            }
        }

        Ok(bindings)
    }

    fn resolve_output(
        &self,
        output: PipelineOutput,
    ) -> Result<ResolvedValue, RenderingPipelineError> {
        let node = self
            .node(output.node_id)
            .ok_or(RenderingPipelineError::MissingNode {
                node_id: output.node_id,
            })?;

        if !node.kind.provides_output(output.socket) {
            return Err(RenderingPipelineError::InvalidOutput {
                node_id: node.id,
                socket: output.socket,
            });
        }

        match node.kind {
            RenderingNodeKind::Texture => {
                let texture_group =
                    node.texture_group
                        .clone()
                        .ok_or(RenderingPipelineError::MissingInput {
                            node_id: node.id,
                            socket: RenderingInputSocket::Texture,
                        })?;

                Ok(ResolvedValue::Texture(texture_group))
            }

            RenderingNodeKind::SplitChannels => {
                let input = PipelineInput {
                    node_id: node.id,
                    socket: RenderingInputSocket::Texture,
                };

                let connection =
                    self.connection_to(input)
                        .ok_or(RenderingPipelineError::MissingInput {
                            node_id: node.id,
                            socket: RenderingInputSocket::Texture,
                        })?;

                let value = self.resolve_output(connection.from)?;

                let ResolvedValue::Texture(texture_group) = value else {
                    return Err(RenderingPipelineError::InvalidConnection {
                        message: "Split Channels requires a texture input".into(),
                    });
                };

                let channel = match output.socket {
                    RenderingOutputSocket::Red => TextureChannel::Red,
                    RenderingOutputSocket::Green => TextureChannel::Green,
                    RenderingOutputSocket::Blue => TextureChannel::Blue,
                    RenderingOutputSocket::Alpha => TextureChannel::Alpha,

                    RenderingOutputSocket::Texture => {
                        return Err(RenderingPipelineError::InvalidOutput {
                            node_id: node.id,
                            socket: output.socket,
                        });
                    }
                };

                Ok(ResolvedValue::Channel(ChannelBinding {
                    texture_group,
                    channel,
                }))
            }

            RenderingNodeKind::MaterialOutput => Err(RenderingPipelineError::InvalidOutput {
                node_id: node.id,
                socket: output.socket,
            }),
        }
    }
}

// =============================================================================
// Pipeline collection
// =============================================================================

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct RenderingPipelines {
    pub pipelines: HashMap<MeshMaterialRef, RenderingPipeline>,

    #[serde(skip)]
    pub revision: u64,
}

impl RenderingPipelines {
    pub fn pipeline_for(&self, material: &MeshMaterialRef) -> Option<&RenderingPipeline> {
        self.pipelines.get(material)
    }

    pub fn pipeline_for_mut(
        &mut self,
        material: &MeshMaterialRef,
    ) -> Option<&mut RenderingPipeline> {
        self.pipelines.get_mut(material)
    }

    pub fn add_material(&mut self, material: MeshMaterialRef) -> &mut RenderingPipeline {
        let pipeline = self.pipelines.entry(material).or_default();

        pipeline.add_node(RenderingNodeKind::MaterialOutput);

        pipeline
    }
    pub fn mark_changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}

// =============================================================================
// Errors
// =============================================================================

#[derive(Clone, Debug)]
pub enum RenderingPipelineError {
    MissingMaterialOutput,

    MissingNode {
        node_id: PipelineNodeId,
    },

    MissingInput {
        node_id: PipelineNodeId,
        socket: RenderingInputSocket,
    },

    InvalidInput {
        node_id: PipelineNodeId,
        socket: RenderingInputSocket,
    },

    InvalidOutput {
        node_id: PipelineNodeId,
        socket: RenderingOutputSocket,
    },

    TypeMismatch {
        input: PipelineInput,
        output: PipelineOutput,
    },

    InvalidConnection {
        message: String,
    },

    Cycle {
        node_id: PipelineNodeId,
    },
}

impl fmt::Display for RenderingPipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMaterialOutput => {
                write!(f, "Missing material output node")
            }

            Self::MissingNode { node_id } => {
                write!(f, "Missing node {:?}", node_id)
            }

            Self::MissingInput { node_id, socket } => {
                write!(
                    f,
                    "Missing input on node {:?}, socket {:?}",
                    node_id, socket
                )
            }

            Self::InvalidInput { node_id, socket } => {
                write!(
                    f,
                    "Invalid input on node {:?}, socket {:?}",
                    node_id, socket
                )
            }

            Self::InvalidOutput { node_id, socket } => {
                write!(
                    f,
                    "Invalid output on node {:?}, socket {:?}",
                    node_id, socket
                )
            }

            Self::TypeMismatch { input, output } => {
                write!(
                    f,
                    "Type mismatch: {:?} cannot connect to {:?}",
                    output, input
                )
            }

            Self::InvalidConnection { message } => {
                write!(f, "Invalid connection: {}", message)
            }

            Self::Cycle { node_id } => {
                write!(f, "Cycle detected at node {:?}", node_id)
            }
        }
    }
}

impl std::error::Error for RenderingPipelineError {}
