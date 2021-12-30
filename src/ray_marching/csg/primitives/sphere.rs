use crate::ray_marching::csg::builder::{CSGCommandBufferBuilder, CSGCommandType};
use crate::ray_marching::csg::CSGNode;

pub struct Sphere {
    // TODO: Remove center in favor of just adding a Translation node
    pub(crate) center: [f32; 3],
    pub(crate) radius: f32,
}

impl CSGNode for Sphere {
    fn build_commands(&self, builder: &mut CSGCommandBufferBuilder) {
        builder
            .push_command(CSGCommandType::Sphere)
            .push_param_vec3(self.center)
            .push_param_float(self.radius);
    }
}