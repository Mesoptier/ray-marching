use crate::ray_marching::csg::{CSGNode, CSGNodeType};
use crate::ray_marching::csg::builder::CSGNodeBufferBuilder;

pub struct Union {
    pub p1: Box<dyn CSGNode>,
    pub p2: Box<dyn CSGNode>,
}

impl CSGNode for Union {
    fn node_type() -> CSGNodeType {
        CSGNodeType::Union
    }

    fn foo(&self, builder: &mut CSGNodeBufferBuilder) {
        builder.push_node(Self::node_type(), 2);

        self.p1.foo(builder);
        self.p2.foo(builder);
    }
}