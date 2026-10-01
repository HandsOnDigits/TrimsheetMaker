use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fmt};

use crate::mesh::import::MeshMaterialRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PipelineNodeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderingNodeKind {
    Texture,
    SplitChannels,
    MaterialOutput,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelSource {
    pub texture_group: String,
    pub socket: RenderingOutputSocket,
}

pub struct ResolvedTextureSource {
    pub image: Handle<Image>,
    pub channel: RenderingOutputSocket,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderingMaterialResult {
    pub base_color_group: Option<String>,
    pub normal_group: Option<String>,
    pub height_group: Option<ChannelSource>,
    pub ao_group: Option<ChannelSource>,
    pub roughness_group: Option<ChannelSource>,
    pub metallic_group: Option<ChannelSource>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderingOutputSocket {
    Texture,
    Red,
    Green,
    Blue,
    Alpha,
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
    pub texture_group: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RenderingPipeline {
    pub nodes: Vec<PipelineNode>,
    pub connections: Vec<RenderingConnection>,
    next_node_id: u64,
}

impl RenderingPipeline {
    pub fn add_node(&mut self, kind: RenderingNodeKind) -> PipelineNodeId {
        self.next_node_id += 1;
        let id = PipelineNodeId(self.next_node_id);
        self.nodes.push(PipelineNode {
            id,
            kind,
            texture_group: None,
        });
        id
    }

    pub fn remove_node(&mut self, id: PipelineNodeId) {
        self.nodes.retain(|n| n.id != id);
        self.connections
            .retain(|c| c.from.node_id != id && c.to.node_id != id);
    }

    pub fn node(&self, id: PipelineNodeId) -> Option<&PipelineNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: PipelineNodeId) -> Option<&mut PipelineNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn material_output(&self) -> Option<&PipelineNode> {
        self.nodes
            .iter()
            .find(|n| n.kind == RenderingNodeKind::MaterialOutput)
    }

    pub fn connect(&mut self, from: PipelineOutput, to: PipelineInput) -> bool {
        self.disconnect(&to);
        self.connections.push(RenderingConnection { from, to });
        true
    }

    pub fn disconnect(&mut self, to: &PipelineInput) -> bool {
        let len_before = self.connections.len();
        self.connections.retain(|c| &c.to != to);
        self.connections.len() < len_before
    }

    pub fn disconnect_output(&mut self, from: &PipelineOutput) -> usize {
        let len_before = self.connections.len();
        self.connections.retain(|c| &c.from != from);
        len_before - self.connections.len()
    }

    pub fn connection_to(&self, input: &PipelineInput) -> Option<&RenderingConnection> {
        self.connections.iter().find(|c| &c.to == input)
    }

    pub fn evaluate(&self) -> Result<RenderingMaterialResult, RenderingPipelineError> {
        let material_node = self
            .material_output()
            .ok_or(RenderingPipelineError::MissingNode {
                node_id: PipelineNodeId(0),
            })?;

        let mut result = RenderingMaterialResult::default();

        for &socket in RenderingNodeKind::MaterialOutput.input_sockets() {
            let input = PipelineInput {
                node_id: material_node.id,
                socket,
            };

            if let Some(conn) = self.connection_to(&input) {
                let source_node =
                    self.node(conn.from.node_id)
                        .ok_or(RenderingPipelineError::MissingNode {
                            node_id: conn.from.node_id,
                        })?;

                let (source_group, source_socket) = match source_node.kind {
                    RenderingNodeKind::Texture => {
                        (source_node.texture_group.clone(), conn.from.socket)
                    }
                    RenderingNodeKind::SplitChannels => {
                        let split_input = PipelineInput {
                            node_id: source_node.id,
                            socket: RenderingInputSocket::Texture,
                        };
                        let group = if let Some(tex_conn) = self.connection_to(&split_input) {
                            self.node(tex_conn.from.node_id)
                                .and_then(|n| n.texture_group.clone())
                        } else {
                            None
                        };
                        (group, conn.from.socket)
                    }
                    RenderingNodeKind::MaterialOutput => (None, conn.from.socket),
                };

                if let Some(group) = source_group {
                    let channel_src = ChannelSource {
                        texture_group: group.clone(),
                        socket: source_socket,
                    };
                    match socket {
                        RenderingInputSocket::BaseColor => result.base_color_group = Some(group),
                        RenderingInputSocket::Normal => result.normal_group = Some(group),
                        RenderingInputSocket::Height => result.height_group = Some(channel_src),
                        RenderingInputSocket::AmbientOcclusion => {
                            result.ao_group = Some(channel_src)
                        }
                        RenderingInputSocket::Roughness => {
                            result.roughness_group = Some(channel_src)
                        }
                        RenderingInputSocket::Metallic => result.metallic_group = Some(channel_src),
                        RenderingInputSocket::Texture => {}
                    }
                }
            }
        }

        Ok(result)
    }
}

#[derive(Clone, Debug, Default)]
pub struct RenderingPipelineResults {
    pub materials: HashMap<MeshMaterialRef, RenderingMaterialResult>,
}

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
        self.pipelines.entry(material).or_default()
    }

    pub fn results(&self) -> Result<RenderingPipelineResults, RenderingPipelineError> {
        let mut results = RenderingPipelineResults::default();
        for (mat_ref, pipeline) in &self.pipelines {
            let res = pipeline.evaluate()?;
            results.materials.insert(mat_ref.clone(), res);
        }
        Ok(results)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum RenderingPipelineError {
    MissingNode {
        node_id: PipelineNodeId,
    },
    MissingInput {
        node_id: PipelineNodeId,
        socket: RenderingInputSocket,
    },
    InvalidOutput {
        node_id: PipelineNodeId,
        socket: RenderingOutputSocket,
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
            Self::MissingNode { node_id } => write!(f, "Missing node {:?}", node_id),
            Self::MissingInput { node_id, socket } => {
                write!(
                    f,
                    "Missing input on node {:?}, socket {:?}",
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
            Self::InvalidConnection { message } => write!(f, "Invalid connection: {}", message),
            Self::Cycle { node_id } => write!(f, "Cycle detected at node {:?}", node_id),
        }
    }
}

impl std::error::Error for RenderingPipelineError {}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderingConnectionError {
    InvalidSocket,
    CycleDetected,
}
