use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};

use crate::renderer::FinalImageView;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};
use vulkano::sync::GpuFuture;

#[repr(C)]
#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

pub(crate) struct RenderPassTriangle {
    gfx_queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
}

impl RenderPassTriangle {
    pub(crate) fn new(gfx_queue: Arc<Queue>, output_format: Format) -> Self {
        // Create shader modules
        let vs = vs::load(gfx_queue.device().clone()).expect("failed to create shader module");
        let fs = fs::load(gfx_queue.device().clone()).expect("failed to create shader module");

        // Create render pass
        let render_pass = vulkano::single_pass_renderpass!(
            gfx_queue.device().clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: output_format,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap();

        // Create graphics pipeline
        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(gfx_queue.device().clone())
            .unwrap();

        Self {
            gfx_queue,
            render_pass,
            pipeline,
        }
    }

    pub(crate) fn render<F>(
        &mut self,
        future: F,
        target_image: FinalImageView,
    ) -> Box<dyn GpuFuture>
    where
        F: GpuFuture + 'static,
    {
        // Create framebuffer
        let framebuffer = Framebuffer::start(self.render_pass.clone())
            .add(target_image)
            .unwrap()
            .build()
            .unwrap();

        // Create primary command buffer builder
        let mut builder = AutoCommandBufferBuilder::primary(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Begin render pass
        builder
            .begin_render_pass(
                framebuffer,
                SubpassContents::Inline,
                vec![[0.0, 0.0, 1.0, 1.0].into()],
            )
            .unwrap();

        // TODO: Execute commands

        // End render pass
        builder.end_render_pass().unwrap();

        // Build and execute the command buffer
        let command_buffer = builder.build().unwrap();
        future
            .then_execute(self.gfx_queue.clone(), command_buffer)
            .unwrap()
            .boxed()
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location=0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(location=0) out vec4 f_color;

void main() {
    f_color = vec4(1.0, 1.0, 1.0, 1.0);
}
"
    }
}
