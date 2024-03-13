use crate::ray_marching::csg::builder::{CSGCommandBufferBuilder, CSGCommandType};
use crate::ray_marching::csg::{BuildCommands, CSGNode};

#[derive(Debug, Clone)]
pub struct Subtraction(pub Box<CSGNode>, pub Box<CSGNode>);

impl BuildCommands for Subtraction {
    fn build_commands(&self, builder: &mut CSGCommandBufferBuilder) {
        self.0.build_commands(builder);
        self.1.build_commands(builder);
        builder.push_command(CSGCommandType::Subtraction);
    }
}

impl From<(Box<CSGNode>, Box<CSGNode>)> for Subtraction {
    fn from((lhs, rhs): (Box<CSGNode>, Box<CSGNode>)) -> Self {
        Subtraction(lhs, rhs)
    }
}
