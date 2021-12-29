use std::borrow::Borrow;

use crate::ray_marching::csg::{CSGNode, CSGNodeType};
use crate::ray_marching::csg::builder::CSGNodeBufferBuilder;

pub struct Sphere {
    // TODO: Remove center in favor of just adding a Translation node
    pub(crate) center: [f32; 3],
    pub(crate) radius: f32,
}

impl CSGNode for Sphere {
    fn node_type() -> CSGNodeType {
        CSGNodeType::Sphere
    }

    fn foo(&self, builder: &mut CSGNodeBufferBuilder) {
        builder
            .push_node(Self::node_type(), 0)
            .push_param_vec3(self.center)
            .push_param_float(self.radius);
    }
}