use std::{borrow::Cow, collections::HashMap};

use bevy_egui::egui;

use egui_node_editor::{
    Graph, GraphEditorState, InputId, InputParamKind, NodeId, OutputId,
    traits::{
        DataTypeTrait, NodeDataTrait, NodeTemplateIter, NodeTemplateTrait, UserResponseTrait,
        WidgetValueTrait,
    },
};

use crate::{
    data::TextureGroup,
    mesh::import::MeshMaterialRef,
    pipeline::{
        PipelineInput, PipelineNodeId, PipelineOutput, RenderingConnection, RenderingInputSocket,
        RenderingNodeKind, RenderingOutputSocket, RenderingPipeline,
    },
    ui::EditorUi,
};

fn ensure_material_output_in_graph(state: &mut RenderingWindowState) -> NodeId {
    // Check if Material Output node already exists in the graph
    for node_id in state.editor.graph.iter_nodes() {
        if state.editor.graph[node_id].user_data.kind == RenderingNodeKind::MaterialOutput {
            return node_id;
        }
    }

    // Spawn a Material Output node if missing
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

fn ensure_rendering_graph(editor_ui: &mut EditorUi, atlas: &crate::data::Atlas) -> bool {
    let material_ref = editor_ui
        .selected_material
        .and_then(|index| atlas.materials.get(index))
        .and_then(|material| material.material_ref.clone());

    let material_changed = match (
        &editor_ui.rendering_window_state.loaded_material,
        &material_ref,
    ) {
        (Some(loaded), Some(current)) => loaded.id != current.id,
        (None, Some(_)) => true,
        (Some(_), None) => true,
        (None, None) => false,
    };

    let has_material_output = editor_ui
        .rendering_window_state
        .editor
        .graph
        .iter_nodes()
        .any(|id| {
            editor_ui.rendering_window_state.editor.graph[id]
                .user_data
                .kind
                == RenderingNodeKind::MaterialOutput
        });

    if material_changed || !has_material_output {
        rebuild_rendering_graph(editor_ui, material_ref.as_ref());
        return true;
    }

    false
}

fn rebuild_rendering_graph(editor_ui: &mut EditorUi, material_ref: Option<&MeshMaterialRef>) {
    clear_rendering_graph(&mut editor_ui.rendering_window_state);

    let Some(material_ref) = material_ref else {
        // If no material is selected, still spawn a Material Output node so the graph isn't empty
        ensure_material_output_in_graph(&mut editor_ui.rendering_window_state);
        return;
    };

    if let Some(pipeline) = editor_ui.rendering_pipelines.pipeline_for_mut(material_ref) {
        if pipeline.material_output().is_none() {
            pipeline.add_node(RenderingNodeKind::MaterialOutput);
        }
    }

    let Some(pipeline) = editor_ui
        .rendering_pipelines
        .pipeline_for(material_ref)
        .cloned()
    else {
        // If pipeline storage isn't initialized yet, fallback to spawning Material Output
        ensure_material_output_in_graph(&mut editor_ui.rendering_window_state);
        editor_ui.rendering_window_state.loaded_material = Some(material_ref.clone());
        return;
    };

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

    ensure_material_output_in_graph(&mut editor_ui.rendering_window_state);

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

    editor_ui.rendering_window_state.loaded_material = Some(material_ref.clone());
}

// -----------------------------------------------------------------------------
// Data types
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Node data
// -----------------------------------------------------------------------------

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
        // Disallow deletion of the Material Output node in the UI
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

fn sync_pipeline_connections(
    graph: &Graph<RenderingNodeData, RenderingDataType, RenderingValue>,
    graph_state: &RenderingGraphState,
    pipeline: &mut RenderingPipeline,
    node_ids: &HashMap<NodeId, PipelineNodeId>,
) -> bool {
    let mut connections = Vec::new();

    for (input_id, output_id) in graph.iter_connections() {
        let Some(input) = graph.try_get_input(input_id) else {
            continue;
        };

        let Some(output) = graph.try_get_output(output_id) else {
            continue;
        };

        let Some(&input_node_id) = node_ids.get(&input.node()) else {
            continue;
        };

        let Some(&output_node_id) = node_ids.get(&output.node) else {
            continue;
        };

        let Some(&input_socket) = graph_state.input_sockets.get(&input_id) else {
            continue;
        };

        let Some(&output_socket) = graph_state.output_sockets.get(&output_id) else {
            continue;
        };

        connections.push(RenderingConnection {
            from: PipelineOutput {
                node_id: output_node_id,
                socket: output_socket,
            },
            to: PipelineInput {
                node_id: input_node_id,
                socket: input_socket,
            },
        });
    }

    if pipeline.connections == connections {
        return false;
    }

    pipeline.connections = connections;

    true
}

fn sync_texture_groups(
    pipeline: &mut RenderingPipeline,
    node_ids: &HashMap<NodeId, PipelineNodeId>,
    selected_texture_groups: &HashMap<NodeId, String>,
) -> bool {
    let mut changed = false;

    for (&ui_node_id, &pipeline_node_id) in node_ids {
        let Some(selected_group) = selected_texture_groups.get(&ui_node_id) else {
            continue;
        };

        let Some(node) = pipeline.node_mut(pipeline_node_id) else {
            continue;
        };

        if node.kind != RenderingNodeKind::Texture {
            continue;
        }

        let new_group = if selected_group.is_empty() {
            None
        } else {
            Some(selected_group.clone())
        };

        if node.texture_group != new_group {
            node.texture_group = new_group;
            changed = true;
        }
    }

    changed
}

// -----------------------------------------------------------------------------
// Values
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// User state
// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct RenderingGraphState {
    pub texture_groups: Vec<TextureGroup>,
    pub selected_texture_groups: HashMap<NodeId, String>,

    pub input_sockets: HashMap<InputId, RenderingInputSocket>,
    pub output_sockets: HashMap<OutputId, RenderingOutputSocket>,
}

// -----------------------------------------------------------------------------
// Node templates
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderingNodeTemplate {
    Texture,
    SplitChannels,
}

impl NodeTemplateIter for RenderingNodeTemplate {
    type Item = Self;

    fn all_kinds(&self) -> Vec<Self::Item> {
        // MaterialOutput is omitted here to prevent creating duplicate Material Output nodes
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

// -----------------------------------------------------------------------------
// Editor state
// -----------------------------------------------------------------------------

pub type RenderingEditorState = GraphEditorState<
    RenderingNodeData,
    RenderingDataType,
    RenderingValue,
    RenderingNodeTemplate,
    RenderingGraphState,
>;

// -----------------------------------------------------------------------------
// Window state
// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct RenderingWindowState {
    pub editor: RenderingEditorState,
    pub graph_state: RenderingGraphState,

    pub pipeline_node_ids: HashMap<NodeId, PipelineNodeId>,

    pub loaded_material: Option<MeshMaterialRef>,
}

// -----------------------------------------------------------------------------
// Window
// -----------------------------------------------------------------------------

fn sync_pipeline_nodes(
    editor_state: &mut RenderingEditorState,
    pipeline: &mut RenderingPipeline,
    node_ids: &mut HashMap<NodeId, PipelineNodeId>,
    graph_state: &mut RenderingGraphState,
) -> bool {
    let mut changed = false;

    // 1. Remove non-MaterialOutput pipeline nodes whose UI nodes no longer exist
    let existing_nodes: Vec<NodeId> = editor_state.graph.iter_nodes().collect();

    let stale_nodes: Vec<(NodeId, PipelineNodeId)> = node_ids
        .iter()
        .filter_map(|(&ui_node_id, &pipeline_node_id)| {
            if existing_nodes.contains(&ui_node_id) {
                None
            } else {
                Some((ui_node_id, pipeline_node_id))
            }
        })
        .collect();

    for (ui_node_id, pipeline_node_id) in stale_nodes {
        node_ids.remove(&ui_node_id);

        pipeline.nodes.retain(|node| {
            node.id != pipeline_node_id || node.kind == RenderingNodeKind::MaterialOutput
        });

        graph_state.selected_texture_groups.remove(&ui_node_id);
        changed = true;
    }

    // 2. Add pipeline nodes for newly spawned UI nodes
    for ui_node_id in editor_state.graph.iter_nodes() {
        if node_ids.contains_key(&ui_node_id) {
            continue;
        }

        let node = &editor_state.graph[ui_node_id];

        if node.user_data.kind == RenderingNodeKind::MaterialOutput {
            if let Some(mo) = pipeline.material_output() {
                node_ids.insert(ui_node_id, mo.id);
            }
            continue;
        }

        let pipeline_node_id = pipeline.add_node(node.user_data.kind);
        node_ids.insert(ui_node_id, pipeline_node_id);
        changed = true;
    }

    // 3. Guarantee that Material Output always exists in both pipeline and UI graph
    let mo_pipeline_id = match pipeline.material_output() {
        Some(mo) => mo.id,
        None => pipeline.add_node(RenderingNodeKind::MaterialOutput),
    };

    let mo_ui_exists = node_ids
        .iter()
        .any(|(&ui_id, &pipe_id)| pipe_id == mo_pipeline_id && existing_nodes.contains(&ui_id));

    if !mo_ui_exists {
        let node_data = RenderingNodeData {
            kind: RenderingNodeKind::MaterialOutput,
        };

        let ui_node_id = editor_state.graph.add_node(
            "Material Output".to_string(),
            node_data.clone(),
            |graph, node_id| {
                node_data.build_node(graph, node_id, graph_state);
            },
        );

        node_ids.insert(ui_node_id, mo_pipeline_id);
        editor_state
            .node_positions
            .insert(ui_node_id, egui::pos2(500.0, 200.0));
        editor_state.node_order.push(ui_node_id);
        changed = true;
    }

    changed
}

pub fn rendering_window(
    ctx: &egui::Context,
    editor_ui: &mut EditorUi,
    atlas: &mut crate::data::Atlas,
) -> bool {
    let mut open = editor_ui.show_rendering_window;

    editor_ui.rendering_window_state.graph_state.texture_groups =
        editor_ui.project_settings.texture_naming.groups.clone();

    let rebuilt = ensure_rendering_graph(editor_ui, atlas);

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

            let response = editor_ui.rendering_window_state.editor.draw_graph_editor(
                ui,
                RenderingNodeTemplate::Texture,
                &mut editor_ui.rendering_window_state.graph_state,
                Vec::new(),
            );

            let material_ref = editor_ui
                .selected_material
                .and_then(|index| atlas.materials.get(index))
                .and_then(|material| material.material_ref.clone());

            if let Some(material_ref) = material_ref {
                if !rebuilt {
                    let editor_state = &mut editor_ui.rendering_window_state.editor;
                    let graph_state = &mut editor_ui.rendering_window_state.graph_state;
                    let node_ids = &mut editor_ui.rendering_window_state.pipeline_node_ids;

                    if let Some(pipeline) = editor_ui
                        .rendering_pipelines
                        .pipeline_for_mut(&material_ref)
                    {
                        if sync_rendering_pipeline(editor_state, graph_state, pipeline, node_ids) {
                            atlas.mark_image_dirty();
                        }
                    }
                }
            }

            let _ = response;
        });

    editor_ui.show_rendering_window = open;

    // Access .response inside InnerResponse to check if pointer is over the window
    window_response
        .map(|inner_response| inner_response.response.contains_pointer())
        .unwrap_or(false)
}

fn sync_rendering_pipeline(
    editor_state: &mut RenderingEditorState,
    graph_state: &mut RenderingGraphState,
    pipeline: &mut RenderingPipeline,
    node_ids: &mut HashMap<NodeId, PipelineNodeId>,
) -> bool {
    let mut changed = false;

    if sync_pipeline_nodes(editor_state, pipeline, node_ids, graph_state) {
        changed = true;
    }

    if sync_texture_groups(pipeline, node_ids, &graph_state.selected_texture_groups) {
        changed = true;
    }

    if sync_pipeline_connections(&editor_state.graph, graph_state, pipeline, node_ids) {
        changed = true;
    }

    changed
}

fn clear_rendering_graph(state: &mut RenderingWindowState) {
    state.editor.graph = Graph::default();

    state.editor.node_positions.clear();
    state.editor.node_order.clear();

    state.graph_state.selected_texture_groups.clear();
    state.graph_state.input_sockets.clear();
    state.graph_state.output_sockets.clear();

    state.pipeline_node_ids.clear();

    state.loaded_material = None;
}

fn find_output_id(
    graph: &Graph<RenderingNodeData, RenderingDataType, RenderingValue>,
    graph_state: &RenderingGraphState,
    node_id: NodeId,
    socket: RenderingOutputSocket,
) -> Option<OutputId> {
    graph[node_id]
        .output_ids()
        .find(|output_id| graph_state.output_sockets.get(output_id) == Some(&socket))
}

fn find_input_id(
    graph: &Graph<RenderingNodeData, RenderingDataType, RenderingValue>,
    graph_state: &RenderingGraphState,
    node_id: NodeId,
    socket: RenderingInputSocket,
) -> Option<InputId> {
    graph[node_id]
        .input_ids()
        .find(|input_id| graph_state.input_sockets.get(input_id) == Some(&socket))
}
