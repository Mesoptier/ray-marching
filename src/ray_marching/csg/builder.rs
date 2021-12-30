#[derive(Debug)]
#[repr(u32)]
pub enum CSGCommandType {
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
    PopTransform,
}

#[derive(Debug)]
pub struct CSGCommandDescriptor {
    cmd_type: CSGCommandType,
    param_offset: u32,
}

pub struct CSGCommandBufferBuilder {
    pub commands: Vec<CSGCommandDescriptor>,
    pub params: Vec<u32>,
}

impl CSGCommandBufferBuilder {
    pub fn new() -> Self {
        Self {
            commands: vec![],
            params: vec![],
        }
    }

    /// Push a command onto the command stack.
    /// Must be called before pushing the command parameters.
    pub fn push_command(&mut self, cmd_type: CSGCommandType) -> &mut Self {
        self.commands.push(CSGCommandDescriptor {
            cmd_type,
            param_offset: self.params.len() as u32,
        });
        self
    }

    /// Push a GLSL vec3 param onto the parameter stack.
    /// Must be called after pushing the command.
    pub fn push_param_vec3(&mut self, value: [f32; 3]) -> &mut Self {
        for v in value {
            self.params.push(v.to_bits());
        }
        self
    }

    /// Push a GLSL float param onto the parameter stack.
    /// Must be called after pushing the command.
    pub fn push_param_float(&mut self, value: f32) -> &mut Self {
        self.params.push(value.to_bits());
        self
    }
}