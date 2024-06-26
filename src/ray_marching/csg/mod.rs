use std::collections::HashMap;

use egui_node_graph::InputParamKind;
use enum_dispatch::enum_dispatch;

pub use operations::*;
pub use primitives::*;

use crate::csg_node_graph::ValueType;
use crate::ray_marching::csg::builder::CSGCommandBufferBuilder;

pub(crate) mod builder;
mod operations;
mod primitives;

#[enum_dispatch]
pub trait BuildCommands {
    fn build_commands(&self, builder: &mut CSGCommandBufferBuilder);
}

#[enum_dispatch]
pub trait CSGNodeTemplateTrait {
    fn name(&self) -> &'static str;
    fn input_params(&self) -> Vec<(&'static str, ValueType, InputParamKind)>;
    fn evaluate(&self, input_params: HashMap<String, ValueType>) -> Option<CSGNode>;
}

#[enum_dispatch(BuildCommands)]
#[derive(Debug, Clone)]
pub enum CSGNode {
    // Primitives
    Sphere,
    Box,
    // Plane,

    // Binary operators
    Union,
    Subtraction,
    // Intersection,

    // Space transformations
    // Translation,
    // Rotation,
    // Scale,
}

#[enum_dispatch(CSGNodeTemplateTrait)]
#[derive(Debug, Clone)]
pub enum CSGNodeTemplate {
    Sphere(SphereTemplate),
    Box(BoxTemplate),
    Union(UnionTemplate),
    Subtraction(SubtractionTemplate),
}

impl CSGNodeTemplate {
    pub fn all() -> impl IntoIterator<Item = Self> {
        [
            CSGNodeTemplate::Sphere(SphereTemplate),
            CSGNodeTemplate::Box(BoxTemplate),
            CSGNodeTemplate::Union(UnionTemplate),
            CSGNodeTemplate::Subtraction(SubtractionTemplate),
        ]
    }
}
