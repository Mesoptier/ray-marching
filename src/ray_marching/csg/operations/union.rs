use crate::ray_marching::csg::builder::{CSGCommandBufferBuilder, CSGCommandType};
use crate::ray_marching::csg::{BuildCommands, CSGNode};

#[derive(Debug, Clone)]
pub struct Union(pub Box<CSGNode>, pub Box<CSGNode>);

impl BuildCommands for Union {
    fn build_commands(&self, builder: &mut CSGCommandBufferBuilder) {
        self.0.build_commands(builder);
        self.1.build_commands(builder);
        builder.push_command(CSGCommandType::Union);
    }
}
