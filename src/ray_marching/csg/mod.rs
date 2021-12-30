use crate::ray_marching::csg::builder::CSGCommandBufferBuilder;

pub(crate) mod builder;
pub(crate) mod operations;
pub(crate) mod primitives;

pub trait CSGNode {
    fn build_commands(&self, builder: &mut CSGCommandBufferBuilder);
}
