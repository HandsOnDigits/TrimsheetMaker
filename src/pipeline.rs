use serde::{Deserialize, Serialize};

pub type PipelineNodeId = u64;

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

        // Every new pipeline always has exactly one Material Output.
        pipeline.add_node(RenderingNodeKind::MaterialOutput);

        pipeline
    }

    pub fn add_node(&mut self, kind: RenderingNodeKind) -> PipelineNodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;

        self.nodes.push(RenderingNode { id, kind });

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
}

impl Default for RenderingPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderingNode {
    pub id: PipelineNodeId,
    pub kind: RenderingNodeKind,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderingConnection {
    pub from: PipelineOutput,
    pub to: PipelineInput,
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
