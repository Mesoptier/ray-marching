use crate::ray_marching::csg::builder::CSGNodeBufferBuilder;

pub(crate) mod primitives;
pub(crate) mod builder;
pub(crate) mod operations;

#[derive(Debug)]
#[repr(u32)]
pub enum CSGNodeType {
    // Primitives
    // (0 children, no space transform)
    Sphere = 0,
    Box,
    Plane,

    // Binary operators
    // (2 children, no space transform)
    Union = 100,
    Subtraction,
    Intersection,

    // Space transformations
    // (1 child, transforms space)
    Translation = 200,
    Rotation,
    Scale,
}

pub trait CSGNode {
    fn node_type() -> CSGNodeType where Self: Sized;

    fn foo(&self, builder: &mut CSGNodeBufferBuilder);
}
