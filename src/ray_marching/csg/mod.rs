use enum_dispatch::enum_dispatch;

use crate::ray_marching::csg::builder::CSGCommandBufferBuilder;
use crate::ray_marching::csg::operations::subtraction::Subtraction;
use crate::ray_marching::csg::operations::union::Union;
use crate::ray_marching::csg::primitives::sphere::Sphere;

pub(crate) mod builder;
pub(crate) mod operations;
pub(crate) mod primitives;

#[enum_dispatch]
pub trait BuildCommands {
    fn build_commands(&self, builder: &mut CSGCommandBufferBuilder);
}

#[enum_dispatch(BuildCommands)]
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
