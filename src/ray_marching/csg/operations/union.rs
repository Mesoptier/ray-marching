use crate::ray_marching::csg::builder::{CSGCommandBufferBuilder, CSGCommandType};
use crate::ray_marching::csg::CSGNode;

pub struct Union {
    pub p1: Box<dyn CSGNode>,
    pub p2: Box<dyn CSGNode>,
}

impl CSGNode for Union {
    fn build_commands(&self, builder: &mut CSGCommandBufferBuilder) {
        self.p1.build_commands(builder);
        self.p2.build_commands(builder);
        builder.push_command(CSGCommandType::Union);
    }
}