use enum_dispatch::enum_dispatch;

use crate::ray_marching::csg::builder::CSGCommandBufferBuilder;

pub(crate) mod builder;
mod operations;
mod primitives;

pub use operations::*;
pub use primitives::*;

#[enum_dispatch]
pub trait BuildCommands {
    fn build_commands(&self, builder: &mut CSGCommandBufferBuilder);
}

#[enum_dispatch(BuildCommands)]
#[derive(Debug, Clone)]
pub enum CSGNode {
    // Primitives
    Sphere,
    // Box,
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
