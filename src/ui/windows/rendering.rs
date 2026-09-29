use std::{borrow::Cow, collections::HashMap};

use bevy_egui::egui;
use egui_node_editor::{
    Graph, GraphEditorState, InputParamKind, NodeId,
    traits::{
        CategoryTrait, DataTypeTrait, NodeDataTrait, NodeTemplateIter, NodeTemplateTrait,
        UserResponseTrait, WidgetValueTrait,
    },
};

use crate::{data::TextureGroup, ui::EditorUi};

use super::FloatingWindowInteraction;

// -----------------------------------------------------------------------------
// Data types
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderingDataType {
    Texture,
    Color,
    Normal,
    Height,
    Channel,
}

impl DataTypeTrait<RenderingGraphState> for RenderingDataType {
    fn data_type_color(&self, _user_state: &mut RenderingGraphState) -> egui::Color32 {
        match self {
            Self::Texture => egui::Color32::from_rgb(180, 180, 180),
            Self::Color => egui::Color32::from_rgb(220, 170, 80),
            Self::Channel => egui::Color32::from_rgb(120, 180, 220),
            Self::Normal => egui::Color32::from_rgb(160, 120, 220),
            Self::Height => egui::Color32::from_rgb(180, 140, 100),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(match self {
            Self::Texture => "Texture",
            Self::Color => "Color",
            Self::Channel => "Channel",
            Self::Normal => "Normal",
            Self::Height => "Height",
        })
    }
}

// -----------------------------------------------------------------------------
// Node data
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderingNodeKind {
    Texture,
    SplitChannels,
    MaterialOutput,
}

#[derive(Clone, Debug)]
pub struct RenderingNodeData {
    pub kind: RenderingNodeKind,
}

impl NodeDataTrait for RenderingNodeData {
    type Response = RenderingResponse;
    type UserState = RenderingGraphState;
    type DataType = RenderingDataType;
    type ValueType = RenderingValue;

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
                "Base Color" => RenderingDataType::Color,
                "Normal" => RenderingDataType::Normal,
                "Height" => RenderingDataType::Height,
                "AO" | "Roughness" | "Metallic" => RenderingDataType::Channel,
                _ => return Vec::new(),
            },

            RenderingNodeKind::Texture => return Vec::new(),
        };

        ui.label(egui::RichText::new(data_type.name()).small().weak());

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
                "Base Color" => RenderingDataType::Color,
                "Normal" => RenderingDataType::Normal,
                "Height" => RenderingDataType::Height,
                "AO" | "Roughness" | "Metallic" => RenderingDataType::Channel,
                _ => return Vec::new(),
            },

            RenderingNodeKind::Texture => return Vec::new(),
        };

        ui.label(egui::RichText::new(data_type.name()).small().weak());

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
}

// -----------------------------------------------------------------------------
// Categories
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderingCategory {
    Input,
    Processing,
    Output,
}

impl CategoryTrait for RenderingCategory {
    fn name(&self) -> String {
        match self {
            Self::Input => "Input".to_string(),
            Self::Processing => "Processing".to_string(),
            Self::Output => "Output".to_string(),
        }
    }
}

// -----------------------------------------------------------------------------
// Node templates
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderingNodeTemplate {
    Texture,
    SplitChannels,
    MaterialOutput,
}

impl NodeTemplateIter for RenderingNodeTemplate {
    type Item = Self;

    fn all_kinds(&self) -> Vec<Self> {
        vec![Self::Texture, Self::SplitChannels, Self::MaterialOutput]
    }
}

impl NodeTemplateTrait for RenderingNodeTemplate {
    type NodeData = RenderingNodeData;
    type DataType = RenderingDataType;
    type ValueType = RenderingValue;
    type UserState = RenderingGraphState;
    type CategoryType = RenderingCategory;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        Cow::Borrowed(match self {
            Self::Texture => "Texture",
            Self::SplitChannels => "Split Channels",
            Self::MaterialOutput => "Material Output",
        })
    }

    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<Self::CategoryType> {
        vec![match self {
            Self::Texture => RenderingCategory::Input,
            Self::SplitChannels => RenderingCategory::Processing,
            Self::MaterialOutput => RenderingCategory::Output,
        }]
    }

    fn node_graph_label(&self, _user_state: &mut Self::UserState) -> String {
        self.node_finder_label(_user_state).into_owned()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        RenderingNodeData {
            kind: match self {
                Self::Texture => RenderingNodeKind::Texture,
                Self::SplitChannels => RenderingNodeKind::SplitChannels,
                Self::MaterialOutput => RenderingNodeKind::MaterialOutput,
            },
        }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        match self {
            Self::Texture => {
                graph.add_output_param(node_id, "Texture".to_string(), RenderingDataType::Texture);
            }

            Self::SplitChannels => {
                // INPUT
                graph.add_input_param(
                    node_id,
                    "Texture".to_string(),
                    RenderingDataType::Texture,
                    RenderingValue::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );

                // OUTPUTS
                for channel in ["R", "G", "B", "A"] {
                    graph.add_output_param(
                        node_id,
                        channel.to_string(),
                        RenderingDataType::Channel,
                    );
                }
            }

            Self::MaterialOutput => {
                // INPUTS
                graph.add_input_param(
                    node_id,
                    "Base Color".to_string(),
                    RenderingDataType::Color,
                    RenderingValue::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );

                graph.add_input_param(
                    node_id,
                    "Normal".to_string(),
                    RenderingDataType::Normal,
                    RenderingValue::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );

                graph.add_input_param(
                    node_id,
                    "Height".to_string(),
                    RenderingDataType::Height,
                    RenderingValue::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );

                graph.add_input_param(
                    node_id,
                    "AO".to_string(),
                    RenderingDataType::Channel,
                    RenderingValue::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );

                graph.add_input_param(
                    node_id,
                    "Roughness".to_string(),
                    RenderingDataType::Channel,
                    RenderingValue::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );

                graph.add_input_param(
                    node_id,
                    "Metallic".to_string(),
                    RenderingDataType::Channel,
                    RenderingValue::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );
            }
        }
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
}

// -----------------------------------------------------------------------------
// Window
// -----------------------------------------------------------------------------

pub fn rendering_window(ctx: &egui::Context, editor_ui: &mut EditorUi) -> bool {
    let mut open = editor_ui.show_rendering_window;

    editor_ui.rendering_window_state.graph_state.texture_groups =
        editor_ui.project_settings.texture_naming.groups.clone();

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

    let owns_pointer = window_response
        .as_ref()
        .map(|response| response.owns_pointer(ctx))
        .unwrap_or(false);

    editor_ui.show_rendering_window = open;

    owns_pointer
}
