use egui_node_graph::InputParamKind;

use crate::csg_node_graph::ValueType;
use crate::ray_marching::csg::builder::{CSGCommandBufferBuilder, CSGCommandType};
use crate::ray_marching::csg::{BuildCommands, CSGNode, CSGNodeTemplateTrait};

macro_rules! impl_binary_operation {
    ($name:ident, $template_name:ident, $command:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name(pub Box<CSGNode>, pub Box<CSGNode>);

        impl BuildCommands for $name {
            fn build_commands(&self, builder: &mut CSGCommandBufferBuilder) {
                self.0.build_commands(builder);
                self.1.build_commands(builder);
                builder.push_command(CSGCommandType::$command);
            }
        }

        #[derive(Debug, Clone)]
        pub struct $template_name;

        impl CSGNodeTemplateTrait for $template_name {
            fn name(&self) -> &'static str {
                stringify!($name)
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

            fn evaluate(
                &self,
                input_params: std::collections::HashMap<String, ValueType>,
            ) -> Option<CSGNode> {
                let lhs = input_params.get("A").unwrap().to_csg_node()?;
                let rhs = input_params.get("B").unwrap().to_csg_node()?;
                Some($name(Box::new(lhs), Box::new(rhs)).into())
            }
        }
    };
}

impl_binary_operation!(Union, UnionTemplate, Union);
impl_binary_operation!(Subtraction, SubtractionTemplate, Subtraction);
