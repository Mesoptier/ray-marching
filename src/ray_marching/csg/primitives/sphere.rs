use egui_node_graph::InputParamKind;
use std::collections::HashMap;

use crate::csg_node_graph::ValueType;
use crate::ray_marching::csg::builder::{CSGCommandBufferBuilder, CSGCommandType};
use crate::ray_marching::csg::{BuildCommands, CSGNode, CSGNodeTemplateTrait};

#[derive(Debug, Clone)]
pub struct Sphere {
    // TODO: Remove center in favor of just adding a Translation node
    pub(crate) center: [f32; 3],
    pub(crate) radius: f32,
}

impl BuildCommands for Sphere {
    fn build_commands(&self, builder: &mut CSGCommandBufferBuilder) {
        builder
            .push_command(CSGCommandType::Sphere)
            .push_param_vec3(self.center)
            .push_param_float(self.radius);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SphereTemplate;
impl CSGNodeTemplateTrait for SphereTemplate {
    fn name(&self) -> &'static str {
        "Sphere"
    }

    fn input_params(&self) -> Vec<(&'static str, ValueType, InputParamKind)> {
        vec![
            (
                "center",
                ValueType::Vec3([0.; 3]),
                InputParamKind::ConnectionOrConstant,
            ),
            (
                "radius",
                ValueType::Scalar(1.),
                InputParamKind::ConnectionOrConstant,
            ),
        ]
    }

    fn evaluate(&self, input_params: HashMap<String, ValueType>) -> Option<CSGNode> {
        let center = input_params.get("center").unwrap().to_vec3().unwrap();
        let radius = input_params.get("radius").unwrap().to_scalar().unwrap();
        Some(Sphere { center, radius }.into())
    }
}
