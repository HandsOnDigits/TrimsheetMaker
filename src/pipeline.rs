use std::collections::HashSet;

use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::mesh::import::MeshMaterialRef;

pub type PipelineNodeId = u64;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderingConnection {
    pub from: PipelineOutput,
    pub to: PipelineInput,
}

#[derive(Clone, Debug)]
pub enum RenderingConnectionError {
    MissingNode {
        node_id: PipelineNodeId,
    },

    InvalidOutput {
        node_id: PipelineNodeId,
        socket: RenderingOutputSocket,
    },

    InvalidInput {
        node_id: PipelineNodeId,
        socket: RenderingInputSocket,
    },

    InputAlreadyConnected {
        node_id: PipelineNodeId,
        socket: RenderingInputSocket,
    },

    SelfConnection {
        node_id: PipelineNodeId,
    },
}

#[derive(Clone, Debug, Default)]
pub struct RenderingPipelineResults {
    pub materials: Vec<RenderingMaterialResult>,
}

#[derive(Clone, Debug)]
pub struct RenderingMaterialResult {
    pub material: MeshMaterialRef,
    pub result: RenderingPipelineResult,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderingMaterialPipeline {
    pub material: MeshMaterialRef,
    pub pipeline: RenderingPipeline,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RenderingPipelines {
    pub materials: Vec<RenderingMaterialPipeline>,
}

impl RenderingPipelines {
    pub fn pipeline_for(&self, material: &MeshMaterialRef) -> Option<&RenderingPipeline> {
        self.materials
            .iter()
            .find(|entry| entry.material.id == material.id)
            .map(|entry| &entry.pipeline)
    }

    pub fn pipeline_for_mut(
        &mut self,
        material: &MeshMaterialRef,
    ) -> Option<&mut RenderingPipeline> {
        let entry = self
            .materials
            .iter_mut()
            .find(|entry| entry.material.id == material.id)?;

        if entry.pipeline.material_output().is_none() {
            entry.pipeline.add_node(RenderingNodeKind::MaterialOutput);
        }

        Some(&mut entry.pipeline)
    }

    pub fn add_material(&mut self, material: MeshMaterialRef) -> &mut RenderingPipeline {
        if let Some(index) = self
            .materials
            .iter()
            .position(|entry| entry.material.id == material.id)
        {
            return &mut self.materials[index].pipeline;
        }

        let index = self.materials.len();

        self.materials.push(RenderingMaterialPipeline {
            material,
            pipeline: RenderingPipeline::new(),
        });

        &mut self.materials[index].pipeline
    }

    pub fn results(&self) -> Result<RenderingPipelineResults, RenderingPipelineError> {
        let mut results = RenderingPipelineResults::default();

        for entry in &self.materials {
            results.materials.push(RenderingMaterialResult {
                material: entry.material.clone(),
                result: entry.pipeline.results()?,
            });
        }

        Ok(results)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderingPipeline {
    pub nodes: Vec<RenderingNode>,
    pub connections: Vec<RenderingConnection>,
    pub next_node_id: PipelineNodeId,
}

impl RenderingPipeline {
    pub fn new() -> Self {
        let mut pipeline = Self {
            nodes: Vec::new(),
            connections: Vec::new(),
            next_node_id: 0,
        };

        // Every pipeline always has exactly one Material Output.
        pipeline.add_node(RenderingNodeKind::MaterialOutput);

        pipeline
    }

    pub fn connect(
        &mut self,
        from: PipelineOutput,
        to: PipelineInput,
    ) -> Result<(), RenderingConnectionError> {
        let from_node = self
            .node(from.node_id)
            .ok_or(RenderingConnectionError::MissingNode {
                node_id: from.node_id,
            })?;

        let to_node = self
            .node(to.node_id)
            .ok_or(RenderingConnectionError::MissingNode {
                node_id: to.node_id,
            })?;

        if !from_node.kind.output_sockets().contains(&from.socket) {
            return Err(RenderingConnectionError::InvalidOutput {
                node_id: from.node_id,
                socket: from.socket,
            });
        }

        if !to_node.kind.input_sockets().contains(&to.socket) {
            return Err(RenderingConnectionError::InvalidInput {
                node_id: to.node_id,
                socket: to.socket,
            });
        }

        // An input socket can only have one connection.
        if self.connections.iter().any(|connection| {
            connection.to.node_id == to.node_id && connection.to.socket == to.socket
        }) {
            return Err(RenderingConnectionError::InputAlreadyConnected {
                node_id: to.node_id,
                socket: to.socket,
            });
        }

        // A node cannot connect to itself.
        if from.node_id == to.node_id {
            return Err(RenderingConnectionError::SelfConnection {
                node_id: from.node_id,
            });
        }

        self.connections.push(RenderingConnection { from, to });

        Ok(())
    }

    pub fn disconnect(&mut self, to: &PipelineInput) -> bool {
        let old_len = self.connections.len();

        self.connections.retain(|connection| connection.to != *to);

        self.connections.len() != old_len
    }

    pub fn disconnect_output(&mut self, from: &PipelineOutput) -> usize {
        let old_len = self.connections.len();

        self.connections
            .retain(|connection| connection.from != *from);

        old_len - self.connections.len()
    }

    pub fn add_node(&mut self, kind: RenderingNodeKind) -> PipelineNodeId {
        if kind == RenderingNodeKind::MaterialOutput {
            if let Some(node) = self.material_output() {
                return node.id;
            }
        }

        let id = self.next_node_id;
        self.next_node_id += 1;

        self.nodes.push(RenderingNode {
            id,
            kind,
            texture_group: None,
        });

        id
    }

    pub fn node(&self, id: PipelineNodeId) -> Option<&RenderingNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn node_mut(&mut self, id: PipelineNodeId) -> Option<&mut RenderingNode> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }

    pub fn material_output(&self) -> Option<&RenderingNode> {
        self.nodes
            .iter()
            .find(|node| node.kind == RenderingNodeKind::MaterialOutput)
    }

    pub fn connection_to(&self, input: &PipelineInput) -> Option<&RenderingConnection> {
        self.connections
            .iter()
            .find(|connection| connection.to == *input)
    }

    pub fn results(&self) -> Result<RenderingPipelineResult, RenderingPipelineError> {
        let Some(material_output) = self.material_output() else {
            return Err(RenderingPipelineError::MissingMaterialOutput);
        };

        let mut result = RenderingPipelineResult::default();

        for connection in self
            .connections
            .iter()
            .filter(|connection| connection.to.node_id == material_output.id)
        {
            let value = self.evaluate_output(&connection.from, &mut HashSet::new())?;

            let output = match connection.to.socket {
                RenderingInputSocket::BaseColor => &mut result.base_color,
                RenderingInputSocket::Normal => &mut result.normal,
                RenderingInputSocket::Height => &mut result.height,
                RenderingInputSocket::AmbientOcclusion => &mut result.ambient_occlusion,
                RenderingInputSocket::Roughness => &mut result.roughness,
                RenderingInputSocket::Metallic => &mut result.metallic,

                RenderingInputSocket::Texture => {
                    return Err(RenderingPipelineError::InvalidConnection {
                        message: "Material Output cannot receive a Texture input.".to_owned(),
                    });
                }
            };

            *output = Some(value);
        }

        Ok(result)
    }

    fn evaluate_output(
        &self,
        output: &PipelineOutput,
        visiting: &mut HashSet<PipelineNodeId>,
    ) -> Result<RenderingTextureResult, RenderingPipelineError> {
        let Some(node) = self.node(output.node_id) else {
            return Err(RenderingPipelineError::MissingNode {
                node_id: output.node_id,
            });
        };

        if !visiting.insert(node.id) {
            return Err(RenderingPipelineError::Cycle { node_id: node.id });
        }

        let result = self.evaluate_node_output(node, output, visiting);

        visiting.remove(&node.id);

        result
    }

    fn evaluate_node_output(
        &self,
        node: &RenderingNode,
        output: &PipelineOutput,
        visiting: &mut HashSet<PipelineNodeId>,
    ) -> Result<RenderingTextureResult, RenderingPipelineError> {
        match node.kind {
            RenderingNodeKind::Texture => {
                if output.socket != RenderingOutputSocket::Texture {
                    return Err(RenderingPipelineError::InvalidOutput {
                        node_id: node.id,
                        socket: output.socket,
                    });
                }

                Ok(RenderingTextureResult::Texture {
                    source: output.clone(),
                })
            }

            RenderingNodeKind::SplitChannels => {
                let input = self
                    .input_connection(node.id, RenderingInputSocket::Texture)
                    .ok_or(RenderingPipelineError::MissingInput {
                        node_id: node.id,
                        socket: RenderingInputSocket::Texture,
                    })?;

                let source = self.evaluate_output(&input.from, visiting)?;

                match output.socket {
                    RenderingOutputSocket::Red
                    | RenderingOutputSocket::Green
                    | RenderingOutputSocket::Blue
                    | RenderingOutputSocket::Alpha => Ok(RenderingTextureResult::Channel {
                        source: Box::new(source),
                        channel: output.socket,
                    }),

                    RenderingOutputSocket::Texture => Err(RenderingPipelineError::InvalidOutput {
                        node_id: node.id,
                        socket: output.socket,
                    }),
                }
            }

            RenderingNodeKind::MaterialOutput => Err(RenderingPipelineError::InvalidOutput {
                node_id: node.id,
                socket: output.socket,
            }),
        }
    }

    fn input_connection(
        &self,
        node_id: PipelineNodeId,
        socket: RenderingInputSocket,
    ) -> Option<&RenderingConnection> {
        self.connections
            .iter()
            .find(|connection| connection.to.node_id == node_id && connection.to.socket == socket)
    }
}

impl Default for RenderingPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Default)]
pub struct RenderingPipelineResult {
    pub base_color: Option<RenderingTextureResult>,
    pub normal: Option<RenderingTextureResult>,
    pub height: Option<RenderingTextureResult>,
    pub ambient_occlusion: Option<RenderingTextureResult>,
    pub roughness: Option<RenderingTextureResult>,
    pub metallic: Option<RenderingTextureResult>,
}

#[derive(Clone, Debug)]
pub enum RenderingTextureResult {
    Texture {
        source: PipelineOutput,
    },

    Channel {
        source: Box<RenderingTextureResult>,
        channel: RenderingOutputSocket,
    },
}

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderingNode {
    pub id: PipelineNodeId,
    pub kind: RenderingNodeKind,

    // Only used by Texture nodes.
    pub texture_group: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineOutput {
    pub node_id: PipelineNodeId,
    pub socket: RenderingOutputSocket,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineInput {
    pub node_id: PipelineNodeId,
    pub socket: RenderingInputSocket,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderingInputSocket {
    Texture,
    BaseColor,
    Normal,
    Height,
    AmbientOcclusion,
    Roughness,
    Metallic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderingOutputSocket {
    Texture,
    Red,
    Green,
    Blue,
    Alpha,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_connects_to_base_color() {
        let mut pipeline = RenderingPipeline::new();

        let texture = pipeline.add_node(RenderingNodeKind::Texture);
        let output = pipeline.material_output().unwrap().id;

        pipeline
            .connect(
                PipelineOutput {
                    node_id: texture,
                    socket: RenderingOutputSocket::Texture,
                },
                PipelineInput {
                    node_id: output,
                    socket: RenderingInputSocket::BaseColor,
                },
            )
            .unwrap();

        let result = pipeline.results().unwrap();

        assert!(matches!(
            result.base_color,
            Some(RenderingTextureResult::Texture { .. })
        ));
    }

    #[test]
    fn texture_connects_through_split_channels() {
        let mut pipeline = RenderingPipeline::new();

        let texture = pipeline.add_node(RenderingNodeKind::Texture);
        let split = pipeline.add_node(RenderingNodeKind::SplitChannels);
        let output = pipeline.material_output().unwrap().id;

        pipeline
            .connect(
                PipelineOutput {
                    node_id: texture,
                    socket: RenderingOutputSocket::Texture,
                },
                PipelineInput {
                    node_id: split,
                    socket: RenderingInputSocket::Texture,
                },
            )
            .unwrap();

        pipeline
            .connect(
                PipelineOutput {
                    node_id: split,
                    socket: RenderingOutputSocket::Red,
                },
                PipelineInput {
                    node_id: output,
                    socket: RenderingInputSocket::Roughness,
                },
            )
            .unwrap();

        let result = pipeline.results().unwrap();

        match result.roughness {
            Some(RenderingTextureResult::Channel {
                channel: RenderingOutputSocket::Red,
                ..
            }) => {}

            _ => panic!("Expected Red channel result"),
        }
    }

    #[test]
    fn input_cannot_have_two_connections() {
        let mut pipeline = RenderingPipeline::new();

        let texture_a = pipeline.add_node(RenderingNodeKind::Texture);
        let texture_b = pipeline.add_node(RenderingNodeKind::Texture);
        let output = pipeline.material_output().unwrap().id;

        let input = PipelineInput {
            node_id: output,
            socket: RenderingInputSocket::BaseColor,
        };

        pipeline
            .connect(
                PipelineOutput {
                    node_id: texture_a,
                    socket: RenderingOutputSocket::Texture,
                },
                input.clone(),
            )
            .unwrap();

        let error = pipeline
            .connect(
                PipelineOutput {
                    node_id: texture_b,
                    socket: RenderingOutputSocket::Texture,
                },
                input,
            )
            .unwrap_err();

        assert!(matches!(
            error,
            RenderingConnectionError::InputAlreadyConnected { .. }
        ));
    }
}
