use crate::ray_marching::csg::CSGNodeType;

#[derive(Debug)]
pub struct CSGNodeDescriptor {
    node_type: CSGNodeType,
    param_offset: u32,
    child_count: u32,
}

pub struct CSGNodeBufferBuilder {
    pub nodes: Vec<CSGNodeDescriptor>,
    pub params: Vec<u32>,
}

impl CSGNodeBufferBuilder {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            params: vec![],
        }
    }

    pub fn push_node(&mut self, node_type: CSGNodeType, child_count: u32) -> &mut Self {
        self.nodes.push(CSGNodeDescriptor {
            node_type,
            param_offset: self.params.len() as u32,
            child_count
        });
        self
    }

    pub fn push_param_vec3(&mut self, value: [f32; 3]) -> &mut Self {
        for v in value {
            self.params.push(v.to_bits());
        }
        self
    }

    pub fn push_param_float(&mut self, value: f32) -> &mut Self {
        self.params.push(value.to_bits());
        self
    }
}