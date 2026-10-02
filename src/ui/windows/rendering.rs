use std::{borrow::Cow, collections::HashMap, collections::HashSet};

use bevy::prelude::*;
use bevy_egui::egui;

use egui_node_editor::{
    Graph, GraphEditorState, InputId, InputParamKind, NodeId, OutputId,
    traits::{
        DataTypeTrait, NodeDataTrait, NodeTemplateIter, NodeTemplateTrait, UserResponseTrait,
        WidgetValueTrait,
    },
};

use crate::{
    data::{Atlas, TextureGroup},
    mesh::{MeshPrimitiveSelection, MeshSceneState, import::MeshMaterialRef},
    pipeline::{
        PipelineInput, PipelineNodeId, PipelineOutput, RenderingConnection, RenderingInputSocket,
        RenderingNodeKind, RenderingOutputSocket, RenderingPipeline, RenderingPipelines,
    },
    ui::EditorUi,
};

fn find_output_id(
    graph: &Graph<RenderingNodeData, RenderingDataType, RenderingValue>,
    graph_state: &RenderingGraphState,
    node_id: NodeId,
    socket: RenderingOutputSocket,
) -> Option<OutputId> {
    graph_state
        .output_sockets
        .iter()
        .find(|&(&output_id, &s)| {
            s == socket
                && graph
                    .try_get_output(output_id)
                    .map_or(false, |out| out.node == node_id)
        })
        .map(|(&output_id, _)| output_id)
}

fn find_input_id(
    graph: &Graph<RenderingNodeData, RenderingDataType, RenderingValue>,
    graph_state: &RenderingGraphState,
    node_id: NodeId,
    socket: RenderingInputSocket,
) -> Option<InputId> {
    graph_state
        .input_sockets
        .iter()
        .find(|&(&input_id, &s)| {
            s == socket
                && graph
                    .try_get_input(input_id)
                    .map_or(false, |inp| inp.node == node_id)
        })
        .map(|(&input_id, _)| input_id)
}

fn find_output_socket(
    editor_state: &RenderingWindowState,
    output_id: OutputId,
) -> Option<RenderingOutputSocket> {
    editor_state
        .graph_state
        .output_sockets
        .get(&output_id)
        .copied()
}

fn find_input_socket(
    editor_state: &RenderingWindowState,
    input_id: InputId,
) -> Option<RenderingInputSocket> {
    editor_state
        .graph_state
        .input_sockets
        .get(&input_id)
        .copied()
}

pub fn sync_rendering_pipeline(
    window_state: &mut RenderingWindowState,
    pipeline: &mut RenderingPipeline,
) -> bool {
    let mut changed = false;

    // 1. Sync added nodes (UI graph -> Pipeline)
    let current_ui_nodes: HashSet<_> = window_state.editor.graph.iter_nodes().collect();

    for &ui_node_id in &current_ui_nodes {
        if !window_state.pipeline_node_ids.contains_key(&ui_node_id) {
            let kind = window_state.editor.graph[ui_node_id].user_data.kind;
            let pipe_node_id = pipeline.add_node(kind);
            window_state
                .pipeline_node_ids
                .insert(ui_node_id, pipe_node_id);
            changed = true;
        }
    }

    // 2. Sync deleted nodes (UI graph -> Pipeline)
    let removed_ui_nodes: Vec<_> = window_state
        .pipeline_node_ids
        .keys()
        .copied()
        .filter(|id| !current_ui_nodes.contains(id))
        .collect();

    for ui_node_id in removed_ui_nodes {
        if let Some(pipe_node_id) = window_state.pipeline_node_ids.remove(&ui_node_id) {
            pipeline.remove_node(pipe_node_id);
            window_state
                .graph_state
                .selected_texture_groups
                .remove(&ui_node_id);
            changed = true;
        }
    }

    // 3. Sync UI node settings (e.g. texture group selection)
    for (&ui_node_id, &pipeline_node_id) in window_state.pipeline_node_ids.iter() {
        if let Some(pipeline_node) = pipeline.node_mut(pipeline_node_id) {
            if pipeline_node.kind == RenderingNodeKind::Texture {
                let current_group = window_state
                    .graph_state
                    .selected_texture_groups
                    .get(&ui_node_id)
                    .cloned();
                if pipeline_node.texture_group != current_group {
                    pipeline_node.texture_group = current_group;
                    changed = true;
                }
            }
        }
    }

    // 4. Reconstruct graph connections
    let mut new_connections = Vec::new();

    for (input_id, output_id) in window_state.editor.graph.iter_connections() {
        let Some(input_param) = window_state.editor.graph.try_get_input(input_id) else {
            continue;
        };
        let Some(output_param) = window_state.editor.graph.try_get_output(output_id) else {
            continue;
        };

        let from_ui_node = output_param.node;
        let to_ui_node = input_param.node;

        if let (Some(&from_pipe_id), Some(&to_pipe_id)) = (
            window_state.pipeline_node_ids.get(&from_ui_node),
            window_state.pipeline_node_ids.get(&to_ui_node),
        ) {
            let from_socket = find_output_socket(window_state, output_id);
            let to_socket = find_input_socket(window_state, input_id);

            if let (Some(from_socket), Some(to_socket)) = (from_socket, to_socket) {
                new_connections.push(RenderingConnection {
                    from: PipelineOutput {
                        node_id: from_pipe_id,
                        socket: from_socket,
                    },
                    to: PipelineInput {
                        node_id: to_pipe_id,
                        socket: to_socket,
                    },
                });
            }
        }
    }

    // Deterministic sorting by node ID
    new_connections.sort_by(|a, b| {
        a.from
            .node_id
            .cmp(&b.from.node_id)
            .then_with(|| a.to.node_id.cmp(&b.to.node_id))
    });

    let mut existing_connections = pipeline.connections.clone();
    existing_connections.sort_by(|a, b| {
        a.from
            .node_id
            .cmp(&b.from.node_id)
            .then_with(|| a.to.node_id.cmp(&b.to.node_id))
    });

    if existing_connections != new_connections {
        pipeline.connections = new_connections;
        changed = true;
    }

    changed
}

fn ensure_material_output_in_graph(state: &mut RenderingWindowState) -> NodeId {
    for node_id in state.editor.graph.iter_nodes() {
        if state.editor.graph[node_id].user_data.kind == RenderingNodeKind::MaterialOutput {
            return node_id;
        }
    }

    let node_data = RenderingNodeData {
        kind: RenderingNodeKind::MaterialOutput,
    };

    let ui_node_id = state.editor.graph.add_node(
        "Material Output".to_string(),
        node_data.clone(),
        |graph, node_id| {
            node_data.build_node(graph, node_id, &mut state.graph_state);
        },
    );

    state
        .editor
        .node_positions
        .insert(ui_node_id, egui::pos2(500.0, 200.0));
    state.editor.node_order.push(ui_node_id);

    ui_node_id
}

fn ensure_rendering_graph(
    editor_ui: &mut EditorUi,
    mesh_scene_state: &MeshSceneState,
    selection: Option<MeshPrimitiveSelection>,
    rendering_pipelines: &mut RenderingPipelines,
) -> bool {
    let material_ref =
        selection.and_then(|selection| primitive_material_ref(mesh_scene_state, selection));

    let loaded_material_ref = editor_ui
        .rendering_window_state
        .loaded_material_ref
        .as_ref();

    if loaded_material_ref == material_ref.as_ref() {
        return false;
    }

    rebuild_rendering_graph(editor_ui, material_ref.as_ref(), rendering_pipelines);

    true
}

fn clear_rendering_graph(state: &mut RenderingWindowState) {
    state.editor.graph = Default::default();
    state.editor.node_positions.clear();
    state.editor.node_order.clear();
    state.pipeline_node_ids.clear();
    state.graph_state.input_sockets.clear();
    state.graph_state.output_sockets.clear();
    state.graph_state.selected_texture_groups.clear();
}

fn rebuild_rendering_graph(
    editor_ui: &mut EditorUi,
    material_ref: Option<&MeshMaterialRef>,
    rendering_pipelines: &mut RenderingPipelines,
) {
    clear_rendering_graph(&mut editor_ui.rendering_window_state);

    let Some(material_ref) = material_ref else {
        ensure_material_output_in_graph(&mut editor_ui.rendering_window_state);

        editor_ui.rendering_window_state.loaded_material_ref = None;

        return;
    };

    let pipeline = rendering_pipelines
        .add_material(material_ref.clone())
        .clone();

    let mut pipeline_to_ui = HashMap::<PipelineNodeId, NodeId>::new();

    for node in &pipeline.nodes {
        let node_data = RenderingNodeData { kind: node.kind };

        let label = match node.kind {
            RenderingNodeKind::Texture => "Texture",
            RenderingNodeKind::SplitChannels => "Split Channels",
            RenderingNodeKind::MaterialOutput => "Material Output",
        };

        let ui_node_id = editor_ui.rendering_window_state.editor.graph.add_node(
            label.to_string(),
            node_data.clone(),
            |graph, node_id| {
                node_data.build_node(
                    graph,
                    node_id,
                    &mut editor_ui.rendering_window_state.graph_state,
                );
            },
        );

        pipeline_to_ui.insert(node.id, ui_node_id);

        editor_ui
            .rendering_window_state
            .pipeline_node_ids
            .insert(ui_node_id, node.id);

        editor_ui
            .rendering_window_state
            .editor
            .node_order
            .push(ui_node_id);

        if node.kind == RenderingNodeKind::Texture {
            if let Some(texture_group) = &node.texture_group {
                editor_ui
                    .rendering_window_state
                    .graph_state
                    .selected_texture_groups
                    .insert(ui_node_id, texture_group.clone());
            }
        }

        let position = match node.kind {
            RenderingNodeKind::MaterialOutput => egui::pos2(500.0, 200.0),
            RenderingNodeKind::Texture => egui::pos2(50.0, 100.0),
            RenderingNodeKind::SplitChannels => egui::pos2(275.0, 100.0),
        };

        editor_ui
            .rendering_window_state
            .editor
            .node_positions
            .insert(ui_node_id, position);
    }

    let material_output_ui_node = pipeline
        .material_output()
        .and_then(|node| pipeline_to_ui.get(&node.id))
        .copied();

    if let Some(node_id) = material_output_ui_node {
        editor_ui
            .rendering_window_state
            .editor
            .node_order
            .retain(|id| *id != node_id);

        editor_ui
            .rendering_window_state
            .editor
            .node_order
            .push(node_id);
    }

    for connection in &pipeline.connections {
        let Some(&ui_output_node) = pipeline_to_ui.get(&connection.from.node_id) else {
            continue;
        };

        let Some(&ui_input_node) = pipeline_to_ui.get(&connection.to.node_id) else {
            continue;
        };

        let output_id = find_output_id(
            &editor_ui.rendering_window_state.editor.graph,
            &editor_ui.rendering_window_state.graph_state,
            ui_output_node,
            connection.from.socket,
        );

        let input_id = find_input_id(
            &editor_ui.rendering_window_state.editor.graph,
            &editor_ui.rendering_window_state.graph_state,
            ui_input_node,
            connection.to.socket,
        );

        let (Some(output_id), Some(input_id)) = (output_id, input_id) else {
            continue;
        };

        editor_ui
            .rendering_window_state
            .editor
            .graph
            .add_connection(output_id, input_id);
    }

    editor_ui.rendering_window_state.loaded_material_ref = Some(material_ref.clone());
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderingDataType {
    Texture,
    Channel,
}

impl DataTypeTrait<RenderingGraphState> for RenderingDataType {
    fn data_type_color(&self, _user_state: &mut RenderingGraphState) -> egui::Color32 {
        match self {
            Self::Texture => egui::Color32::from_rgb(180, 180, 180),
            Self::Channel => egui::Color32::from_rgb(120, 180, 220),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(match self {
            Self::Texture => "Texture",
            Self::Channel => "Channel",
        })
    }
}

#[derive(Clone, Debug)]
pub struct RenderingNodeData {
    pub kind: RenderingNodeKind,
}

impl RenderingNodeData {
    pub fn build_node(
        &self,
        graph: &mut Graph<Self, RenderingDataType, RenderingValue>,
        node_id: NodeId,
        graph_state: &mut RenderingGraphState,
    ) {
        match self.kind {
            RenderingNodeKind::Texture => {
                let output = graph.add_output_param(
                    node_id,
                    "Texture".to_string(),
                    RenderingDataType::Texture,
                );

                graph_state
                    .output_sockets
                    .insert(output, RenderingOutputSocket::Texture);
            }

            RenderingNodeKind::SplitChannels => {
                let input = graph.add_input_param(
                    node_id,
                    "Texture".to_string(),
                    RenderingDataType::Texture,
                    RenderingValue::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );

                graph_state
                    .input_sockets
                    .insert(input, RenderingInputSocket::Texture);

                for (name, socket) in [
                    ("R", RenderingOutputSocket::Red),
                    ("G", RenderingOutputSocket::Green),
                    ("B", RenderingOutputSocket::Blue),
                    ("A", RenderingOutputSocket::Alpha),
                ] {
                    let output = graph.add_output_param(
                        node_id,
                        name.to_string(),
                        RenderingDataType::Channel,
                    );

                    graph_state.output_sockets.insert(output, socket);
                }
            }

            RenderingNodeKind::MaterialOutput => {
                for (name, socket) in [
                    ("Base Color", RenderingInputSocket::BaseColor),
                    ("Normal", RenderingInputSocket::Normal),
                    ("Height", RenderingInputSocket::Height),
                    ("AO", RenderingInputSocket::AmbientOcclusion),
                    ("Roughness", RenderingInputSocket::Roughness),
                    ("Metallic", RenderingInputSocket::Metallic),
                ] {
                    let data_type = match socket {
                        RenderingInputSocket::BaseColor | RenderingInputSocket::Normal => {
                            RenderingDataType::Texture
                        }

                        RenderingInputSocket::Height
                        | RenderingInputSocket::AmbientOcclusion
                        | RenderingInputSocket::Roughness
                        | RenderingInputSocket::Metallic => RenderingDataType::Channel,

                        RenderingInputSocket::Texture => unreachable!(),
                    };

                    let input = graph.add_input_param(
                        node_id,
                        name.to_string(),
                        data_type,
                        RenderingValue::None,
                        InputParamKind::ConnectionOnly,
                        true,
                    );

                    graph_state.input_sockets.insert(input, socket);
                }
            }
        }
    }
}

impl NodeDataTrait for RenderingNodeData {
    type Response = RenderingResponse;
    type UserState = RenderingGraphState;
    type DataType = RenderingDataType;
    type ValueType = RenderingValue;

    fn can_delete(
        &self,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> bool {
        self.kind != RenderingNodeKind::MaterialOutput
    }

    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<egui_node_editor::NodeResponse<Self::Response, Self>>
    where
        Self::Response: UserResponseTrait,
    {
        if self.kind == RenderingNodeKind::Texture {
            ui.horizontal(|ui| {
                ui.label("Group");

                let selected = user_state
                    .selected_texture_groups
                    .entry(node_id)
                    .or_default();

                let selected_name = user_state
                    .texture_groups
                    .iter()
                    .find(|group| group.id == *selected)
                    .map(|group| group.name.as_str())
                    .unwrap_or("Select texture group");

                egui::ComboBox::from_id_salt(("texture_group", node_id))
                    .selected_text(selected_name)
                    .show_ui(ui, |ui| {
                        for group in &user_state.texture_groups {
                            ui.selectable_value(selected, group.id.clone(), &group.name);
                        }
                    });
            });
        }

        Vec::new()
    }

    fn output_ui(
        &self,
        ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        param_name: &str,
    ) -> Vec<egui_node_editor::NodeResponse<Self::Response, Self>> {
        let data_type = match self.kind {
            RenderingNodeKind::Texture => RenderingDataType::Texture,
            RenderingNodeKind::SplitChannels => RenderingDataType::Channel,
            RenderingNodeKind::MaterialOutput => return Vec::new(),
        };

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(data_type.name()).small().weak());
            ui.label(param_name);
        });

        Vec::new()
    }
}

#[derive(Clone, Debug, Default)]
pub enum RenderingValue {
    #[default]
    None,
}

impl WidgetValueTrait for RenderingValue {
    type Response = RenderingResponse;
    type UserState = RenderingGraphState;
    type NodeData = RenderingNodeData;

    fn value_widget(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut RenderingGraphState,
        node_data: &RenderingNodeData,
    ) -> Vec<Self::Response> {
        let data_type = match node_data.kind {
            RenderingNodeKind::SplitChannels => RenderingDataType::Texture,

            RenderingNodeKind::MaterialOutput => match param_name {
                "Base Color" => RenderingDataType::Texture,
                "Normal" => RenderingDataType::Texture,
                "Height" | "AO" | "Roughness" | "Metallic" => RenderingDataType::Channel,
                _ => return Vec::new(),
            },

            RenderingNodeKind::Texture => return Vec::new(),
        };

        ui.horizontal(|ui| {
            ui.label(param_name);
            ui.label(egui::RichText::new(data_type.name()).small().weak());
        });

        Vec::new()
    }

    fn value_widget_connected(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut RenderingGraphState,
        node_data: &RenderingNodeData,
    ) -> Vec<Self::Response> {
        let data_type = match node_data.kind {
            RenderingNodeKind::SplitChannels => RenderingDataType::Texture,

            RenderingNodeKind::MaterialOutput => match param_name {
                "Base Color" => RenderingDataType::Texture,
                "Normal" => RenderingDataType::Texture,
                "Height" | "AO" | "Roughness" | "Metallic" => RenderingDataType::Channel,
                _ => return Vec::new(),
            },

            RenderingNodeKind::Texture => return Vec::new(),
        };

        ui.horizontal(|ui| {
            ui.label(param_name);
            ui.label(egui::RichText::new(data_type.name()).small().weak());
        });

        Vec::new()
    }
}

#[derive(Clone, Debug)]
pub enum RenderingResponse {}

impl UserResponseTrait for RenderingResponse {}

#[derive(Default)]
pub struct RenderingGraphState {
    pub texture_groups: Vec<TextureGroup>,
    pub selected_texture_groups: HashMap<NodeId, String>,

    pub input_sockets: HashMap<InputId, RenderingInputSocket>,
    pub output_sockets: HashMap<OutputId, RenderingOutputSocket>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderingNodeTemplate {
    Texture,
    SplitChannels,
}

impl NodeTemplateIter for RenderingNodeTemplate {
    type Item = Self;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![
            RenderingNodeTemplate::Texture,
            RenderingNodeTemplate::SplitChannels,
        ]
    }
}

impl NodeTemplateTrait for RenderingNodeTemplate {
    type NodeData = RenderingNodeData;
    type DataType = RenderingDataType;
    type ValueType = RenderingValue;
    type UserState = RenderingGraphState;
    type CategoryType = ();

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        Cow::Borrowed(match self {
            Self::Texture => "Texture",
            Self::SplitChannels => "Split Channels",
        })
    }

    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<Self::CategoryType> {
        Vec::new()
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into_owned()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        RenderingNodeData {
            kind: match self {
                Self::Texture => RenderingNodeKind::Texture,
                Self::SplitChannels => RenderingNodeKind::SplitChannels,
            },
        }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        let node_data = RenderingNodeData {
            kind: match self {
                Self::Texture => RenderingNodeKind::Texture,
                Self::SplitChannels => RenderingNodeKind::SplitChannels,
            },
        };

        node_data.build_node(graph, node_id, user_state);
    }
}

pub type RenderingEditorState = GraphEditorState<
    RenderingNodeData,
    RenderingDataType,
    RenderingValue,
    RenderingNodeTemplate,
    RenderingGraphState,
>;

#[derive(Default)]
pub struct RenderingWindowState {
    pub editor: RenderingEditorState,
    pub graph_state: RenderingGraphState,

    pub pipeline_node_ids: HashMap<NodeId, PipelineNodeId>,

    pub loaded_material_ref: Option<MeshMaterialRef>,
}

pub fn primitive_material_ref(
    scene_state: &MeshSceneState,
    selection: MeshPrimitiveSelection,
) -> Option<MeshMaterialRef> {
    let mesh = scene_state.get(selection.mesh_id)?;
    let object = mesh.scene.objects.get(selection.object_index)?;
    let primitive = object.primitives.get(selection.primitive_index)?;

    primitive.material.clone()
}

pub fn rendering_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    mesh_scene_state: &MeshSceneState,
    mesh_preview: &crate::mesh::viewport::MeshPreview,
    atlas: &mut Atlas,
    rendering_pipelines: &mut RenderingPipelines,
) -> bool {
    let mut open = editor_ui.show_rendering_window;

    editor_ui.rendering_window_state.graph_state.texture_groups =
        editor_ui.project_settings.texture_naming.groups.clone();

    let selection = mesh_preview.selected_primitive;

    let rebuilt =
        ensure_rendering_graph(editor_ui, mesh_scene_state, selection, rendering_pipelines);

    let window_response = egui::Window::new("Rendering")
        .open(&mut open)
        .default_size([700.0, 500.0])
        .min_size([400.0, 300.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Rendering");
                ui.separator();
                ui.label(egui::RichText::new("Right-click to add a node").weak());
            });

            ui.separator();

            let _ = editor_ui.rendering_window_state.editor.draw_graph_editor(
                ui,
                RenderingNodeTemplate::Texture,
                &mut editor_ui.rendering_window_state.graph_state,
                Vec::new(),
            );
        });

    // The graph editor has now processed this frame's edits.
    //
    // Do not sync the UI graph back into the pipeline during the
    // same frame in which the graph was rebuilt from the pipeline.
    if !rebuilt {
        if let Some(material_ref) =
            selection.and_then(|selection| primitive_material_ref(mesh_scene_state, selection))
        {
            if let Some(pipeline) = rendering_pipelines.pipeline_for_mut(&material_ref) {
                if sync_rendering_pipeline(&mut editor_ui.rendering_window_state, pipeline) {
                    rendering_pipelines.mark_changed();

                    // The rendering graph determines which source textures
                    // are baked into the semantic atlas maps.
                    atlas.mark_image_dirty();
                }
            }
        }
    }

    editor_ui.show_rendering_window = open;

    window_response
        .map(|inner_response| inner_response.response.contains_pointer())
        .unwrap_or(false)
}
