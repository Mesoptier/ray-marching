use std::borrow::Cow;
use std::collections::HashMap;

use egui::{Color32, Ui};
use egui_node_graph::{
    DataTypeTrait, Graph, GraphEditorState, InputId, InputParamKind, NodeDataTrait, NodeId,
    NodeResponse, NodeTemplateIter, NodeTemplateTrait, OutputId, UserResponseTrait,
    WidgetValueTrait,
};

use crate::ray_marching::csg::{CSGNode, Sphere, Subtraction, Union};

pub struct NodeData {
    template: NodeTemplate,
}

#[derive(Eq, PartialEq)]
pub enum DataType {
    Scalar,
    Vec3,
    CSGNode,
}

#[derive(Clone, Debug)]
pub enum ValueType {
    Scalar(f32),
    Vec3([f32; 3]),
    CSGNode(Option<Box<CSGNode>>),
}

impl ValueType {
    fn csg_node(value: impl Into<CSGNode>) -> Self {
        ValueType::CSGNode(Some(Box::new(value.into())))
    }

    fn to_scalar(&self) -> Option<f32> {
        match self {
            ValueType::Scalar(x) => Some(*x),
            _ => None,
        }
    }

    fn to_vec3(&self) -> Option<[f32; 3]> {
        match self {
            ValueType::Vec3(x) => Some(*x),
            _ => None,
        }
    }

    fn to_csg_node(&self) -> Option<CSGNode> {
        match self {
            ValueType::CSGNode(Some(x)) => Some(*x.clone()),
            _ => None,
        }
    }
}

impl Default for ValueType {
    fn default() -> Self {
        ValueType::Scalar(0.0)
    }
}

#[derive(Copy, Clone)]
pub enum NodeTemplate {
    Root,
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
            DataType::CSGNode => "SDF".into(),
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
            NodeTemplate::Root => "Root".into(),
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
            NodeTemplate::Root => {
                graph.add_input_param(
                    node_id,
                    "SDF".to_string(),
                    DataType::CSGNode,
                    ValueType::CSGNode(None),
                    InputParamKind::ConnectionOnly,
                    true,
                );
            }
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
                graph.add_output_param(node_id, "SDF".to_string(), DataType::CSGNode);
            }
            NodeTemplate::Union | NodeTemplate::Subtraction => {
                graph.add_input_param(
                    node_id,
                    "A".to_string(),
                    DataType::CSGNode,
                    ValueType::CSGNode(None),
                    InputParamKind::ConnectionOnly,
                    true,
                );
                graph.add_input_param(
                    node_id,
                    "B".to_string(),
                    DataType::CSGNode,
                    ValueType::CSGNode(None),
                    InputParamKind::ConnectionOnly,
                    true,
                );
                graph.add_output_param(node_id, "SDF".to_string(), DataType::CSGNode);
            }
        }
    }
}

pub struct AllNodeTemplates;
impl NodeTemplateIter for AllNodeTemplates {
    type Item = NodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![
            NodeTemplate::Root,
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
            ValueType::CSGNode(_) => {
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

    pub fn evaluate_root(&self) -> Option<CSGNode> {
        let (_, root_node) = self
            .editor_state
            .graph
            .nodes
            .iter()
            .find(|(_, node)| matches!(node.user_data.template, NodeTemplate::Root))?;
        let input_id = root_node.get_input("SDF").unwrap();
        let mut evaluator = Evaluator::new(&self.editor_state.graph);
        evaluator.evaluate_input(input_id).to_csg_node()
    }
}

struct Evaluator<'a> {
    graph: &'a MyGraph,
    output_cache: HashMap<OutputId, Option<ValueType>>,
}

impl<'a> Evaluator<'a> {
    fn new(graph: &'a MyGraph) -> Self {
        Self {
            graph,
            output_cache: HashMap::new(),
        }
    }

    fn evaluate_input(&mut self, input_id: InputId) -> ValueType {
        self.graph
            .connection(input_id)
            .and_then(|output_id| self.evaluate_output(output_id))
            .unwrap_or_else(|| self.graph.get_input(input_id).value.clone())
    }

    fn evaluate_output(&mut self, output_id: OutputId) -> Option<ValueType> {
        if !self.output_cache.contains_key(&output_id) {
            self.evaluate_node(self.graph.get_output(output_id).node);
        }
        self.output_cache.get(&output_id).cloned().flatten()
    }

    fn evaluate_node(&mut self, node_id: NodeId) {
        let node = &self.graph[node_id];

        match node.user_data.template {
            NodeTemplate::Root => {}
            NodeTemplate::Sphere => {
                let center = self
                    .evaluate_input(node.get_input("center").unwrap())
                    .to_vec3()
                    .unwrap();
                let radius = self
                    .evaluate_input(node.get_input("radius").unwrap())
                    .to_scalar()
                    .unwrap();

                let sdf = ValueType::csg_node(Sphere { center, radius });
                self.output_cache
                    .insert(node.get_output("SDF").unwrap(), Some(sdf));
            }
            NodeTemplate::Union | NodeTemplate::Subtraction => {
                let a = self
                    .evaluate_input(node.get_input("A").unwrap())
                    .to_csg_node();
                let b = self
                    .evaluate_input(node.get_input("B").unwrap())
                    .to_csg_node();

                if let (Some(a), Some(b)) = (a, b) {
                    let a = a.into();
                    let b = b.into();
                    let sdf = match node.user_data.template {
                        NodeTemplate::Union => ValueType::csg_node(Union(a, b)),
                        NodeTemplate::Subtraction => ValueType::csg_node(Subtraction(a, b)),
                        _ => unreachable!(),
                    };
                    self.output_cache
                        .insert(node.get_output("SDF").unwrap(), Some(sdf));
                }
            }
        }
    }
}
