#[derive(Debug)]
#[repr(u32)]
pub enum CSGCommandType {
    // Primitives
    // (0 children, no space transform)
    Sphere = 0,
    Box,
    // Plane,

    // Binary operators
    // (2 children, no space transform)
    Union = 100,
    Subtraction,
    // Intersection,

    // Space transformations
    // (1 child, transforms space)
    // TranslationPush = 200,
    // TranslationPop,
    // RotationPush,
    // RotationPop,
    // ScalePush,
    // ScalePop,
}

pub struct CSGCommandBufferBuilder {
    pub cmd_count: u32,
    pub buffer: Vec<u32>,
}

impl CSGCommandBufferBuilder {
    pub fn new() -> Self {
        Self {
            cmd_count: 0,
            buffer: Vec::new(),
        }
    }

    /// Push a command onto the command stack.
    /// Must be called before pushing the command parameters.
    pub fn push_command(&mut self, cmd_type: CSGCommandType) -> &mut Self {
        self.cmd_count += 1;
        self.buffer.push(cmd_type as u32);
        self
    }

    /// Push a GLSL vec3 param onto the parameter stack.
    /// Must be called after pushing the command.
    pub fn push_param_vec3(&mut self, value: [f32; 3]) -> &mut Self {
        for v in value {
            self.buffer.push(v.to_bits());
        }
        self
    }

    /// Push a GLSL float param onto the parameter stack.
    /// Must be called after pushing the command.
    pub fn push_param_float(&mut self, value: f32) -> &mut Self {
        self.buffer.push(value.to_bits());
        self
    }
}
