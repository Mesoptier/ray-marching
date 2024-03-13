use crate::gui::ValueType;
use egui_node_graph::InputParamKind;
use std::collections::HashMap;
pub use subtraction::*;
pub use union::*;

use crate::ray_marching::csg::{CSGNode, CSGNodeTemplateTrait};

mod subtraction;
mod union;

#[derive(Debug, Clone)]
pub struct BinaryOperationTemplate<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> BinaryOperationTemplate<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Into<CSGNode> + From<(Box<CSGNode>, Box<CSGNode>)>> CSGNodeTemplateTrait
    for BinaryOperationTemplate<T>
{
    fn name(&self) -> &'static str {
        "TODO: BinaryOperationTemplate name()"
    }

    fn input_params(&self) -> Vec<(&'static str, ValueType, InputParamKind)> {
        vec![
            (
                "A",
                ValueType::CSGNode(None),
                InputParamKind::ConnectionOnly,
            ),
            (
                "B",
                ValueType::CSGNode(None),
                InputParamKind::ConnectionOnly,
            ),
        ]
    }

    fn evaluate(&self, input_params: HashMap<String, ValueType>) -> Option<CSGNode> {
        let lhs = input_params.get("A").unwrap().to_csg_node()?;
        let rhs = input_params.get("B").unwrap().to_csg_node()?;
        Some(T::from((Box::new(lhs), Box::new(rhs))).into())
    }
}
