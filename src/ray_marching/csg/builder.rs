use bytemuck::{Pod, Zeroable};
use vulkano::buffer::allocator::SubbufferAllocator;
use vulkano::buffer::Subbuffer;
use vulkano::DeviceSize;

#[derive(Debug)]
#[repr(u32)]
pub enum CSGCommandType {
    // Primitives
    // (0 children, no space transform)
    Sphere = 0,
    // Box,
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

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct CSGCommandDescriptor {
    _cmd_type: u32,
    _param_offset: u32,
}

impl CSGCommandDescriptor {
    pub fn new(cmd_type: CSGCommandType, param_offset: u32) -> Self {
        Self {
            _cmd_type: cmd_type as u32,
            _param_offset: param_offset,
        }
    }
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
        self.commands.push(CSGCommandDescriptor::new(
            cmd_type,
            self.params.len() as u32,
        ));
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

    pub fn build(
        self,
        subbuffer_allocator: &SubbufferAllocator,
    ) -> (u32, Subbuffer<[CSGCommandDescriptor]>, Subbuffer<[u32]>) {
        let cmd_count = self.commands.len() as u32;

        let csg_commands_buffer = subbuffer_allocator
            .allocate_slice(self.commands.len().max(1) as DeviceSize)
            .unwrap();
        if !self.commands.is_empty() {
            csg_commands_buffer
                .write()
                .unwrap()
                .copy_from_slice(&self.commands);
        }

        let csg_params_buffer = subbuffer_allocator
            .allocate_slice(self.params.len().max(1) as DeviceSize)
            .unwrap();
        if !self.params.is_empty() {
            csg_params_buffer
                .write()
                .unwrap()
                .copy_from_slice(&self.params);
        }

        (cmd_count, csg_commands_buffer, csg_params_buffer)
    }
}
