use egui::WidgetType::DragValue;
use egui::{Color32, Ui};
use egui_node_graph::{
    DataTypeTrait, Graph, GraphEditorState, InputParamKind, NodeDataTrait, NodeId, NodeResponse,
    NodeTemplateIter, NodeTemplateTrait, UserResponseTrait, WidgetValueTrait,
};
use std::borrow::Cow;

pub struct NodeData {
    template: NodeTemplate,
}

#[derive(Eq, PartialEq)]
pub enum DataType {
    Scalar,
    Vec3,
    /// Signed Distance Function
    SDF,
}

#[derive(Copy, Clone, Debug)]
pub enum ValueType {
    Scalar(f32),
    Vec3([f32; 3]),
    SDF,
}

impl Default for ValueType {
    fn default() -> Self {
        ValueType::Scalar(0.0)
    }
}

#[derive(Copy, Clone)]
pub enum NodeTemplate {
    Sphere,
    Union,
    Subtraction,
}

#[derive(Copy, Clone, Debug)]
pub struct Response;

#[derive(Default)]
pub struct GraphState;

impl DataTypeTrait<GraphState> for DataType {
    fn data_type_color(&self, user_state: &mut GraphState) -> Color32 {
        // TODO
        Color32::from_rgb(100, 20, 20)
    }

    fn name(&self) -> Cow<str> {
        match self {
            DataType::Scalar => "Scalar".into(),
            DataType::Vec3 => "Vec3".into(),
            DataType::SDF => "SDF".into(),
        }
    }
}

impl NodeTemplateTrait for NodeTemplate {
    type NodeData = NodeData;
    type DataType = DataType;
    type ValueType = ValueType;
    type UserState = GraphState;
    type CategoryType = ();

    fn node_finder_label(&self, user_state: &mut Self::UserState) -> Cow<str> {
        match self {
            NodeTemplate::Sphere => "Sphere".into(),
            NodeTemplate::Union => "Union".into(),
            NodeTemplate::Subtraction => "Subtraction".into(),
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        NodeData { template: *self }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        match self {
            NodeTemplate::Sphere => {
                graph.add_input_param(
                    node_id,
                    "center".to_string(),
                    DataType::Vec3,
                    ValueType::Vec3([0.0; 3]),
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
                graph.add_input_param(
                    node_id,
                    "radius".to_string(),
                    DataType::Scalar,
                    ValueType::Scalar(1.0),
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
                graph.add_output_param(node_id, "SDF".to_string(), DataType::SDF);
            }
            NodeTemplate::Union | NodeTemplate::Subtraction => {
                graph.add_input_param(
                    node_id,
                    "A".to_string(),
                    DataType::SDF,
                    ValueType::SDF,
                    InputParamKind::ConnectionOnly,
                    true,
                );
                graph.add_input_param(
                    node_id,
                    "B".to_string(),
                    DataType::SDF,
                    ValueType::SDF,
                    InputParamKind::ConnectionOnly,
                    true,
                );
                graph.add_output_param(node_id, "SDF".to_string(), DataType::SDF);
            }
        }
    }
}

pub struct AllNodeTemplates;
impl NodeTemplateIter for AllNodeTemplates {
    type Item = NodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![
            NodeTemplate::Sphere,
            NodeTemplate::Union,
            NodeTemplate::Subtraction,
        ]
    }
}

impl WidgetValueTrait for ValueType {
    type Response = Response;
    type UserState = GraphState;
    type NodeData = NodeData;

    fn value_widget(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut Ui,
        _user_state: &mut Self::UserState,
        _node_data: &Self::NodeData,
    ) -> Vec<Self::Response> {
        match self {
            ValueType::Scalar(value) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(egui::DragValue::new(value))
                });
            }
            ValueType::Vec3(value) => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut value[0]));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut value[1]));
                    ui.label("z");
                    ui.add(egui::DragValue::new(&mut value[2]));
                });
            }
            ValueType::SDF => {
                ui.label(param_name);
            }
        }
        Vec::default()
    }
}

impl UserResponseTrait for Response {}
impl NodeDataTrait for NodeData {
    type Response = Response;
    type UserState = GraphState;
    type DataType = DataType;
    type ValueType = ValueType;

    fn bottom_ui(
        &self,
        ui: &mut Ui,
        node_id: NodeId,
        graph: &Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, Self>>
    where
        Self::Response: UserResponseTrait,
    {
        Vec::default()
    }
}

type MyGraph = Graph<NodeData, DataType, ValueType>;
type MyEditorState = GraphEditorState<NodeData, DataType, ValueType, NodeTemplate, GraphState>;

#[derive(Default)]
pub struct Gui {
    pub editor_state: MyEditorState,
    user_state: GraphState,
}

impl Gui {
    pub fn draw(&mut self, ui: &mut Ui) {
        let _graph_response = self.editor_state.draw_graph_editor(
            ui,
            AllNodeTemplates,
            &mut self.user_state,
            Vec::default(),
        );
    }
}
