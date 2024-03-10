// Copyright (c) 2021 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferInheritanceInfo, CommandBufferUsage,
    SecondaryAutoCommandBuffer,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Queue;
use vulkano::image::sampler::{Sampler, SamplerCreateInfo};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
    PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::Subpass;

/// Vertex for textured quads
#[repr(C)]
#[derive(BufferContents, Vertex)]
pub struct TexturedVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
    #[format(R32G32_SFLOAT)]
    pub tex_coords: [f32; 2],
}

pub fn textured_quad(width: f32, height: f32) -> (Vec<TexturedVertex>, Vec<u32>) {
    (
        vec![
            TexturedVertex {
                position: [-(width / 2.0), -(height / 2.0)],
                tex_coords: [0.0, 1.0],
            },
            TexturedVertex {
                position: [-(width / 2.0), height / 2.0],
                tex_coords: [0.0, 0.0],
            },
            TexturedVertex {
                position: [width / 2.0, height / 2.0],
                tex_coords: [1.0, 0.0],
            },
            TexturedVertex {
                position: [width / 2.0, -(height / 2.0)],
                tex_coords: [1.0, 1.0],
            },
        ],
        vec![0, 2, 1, 0, 3, 2],
    )
}

/// A subpass pipeline that fills a quad over frame
pub struct PixelsDrawPipeline {
    gfx_queue: Arc<Queue>,
    subpass: Subpass,
    pipeline: Arc<GraphicsPipeline>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    vertices: Subbuffer<[TexturedVertex]>,
    indices: Subbuffer<[u32]>,
}

impl PixelsDrawPipeline {
    pub fn new(
        gfx_queue: Arc<Queue>,
        subpass: Subpass,
        memory_allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> PixelsDrawPipeline {
        let (vertices, indices) = textured_quad(2.0, 2.0);
        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.into_iter(),
        )
        .unwrap();
        let index_buffer = Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices.into_iter(),
        )
        .unwrap();

        let pipeline = {
            let device = gfx_queue.device().clone();

            let vs = vs::load(device.clone())
                .expect("failed to create shader module")
                .entry_point("main")
                .unwrap();
            let fs = fs::load(device.clone())
                .expect("failed to create shader module")
                .entry_point("main")
                .unwrap();

            let vertex_input_state = TexturedVertex::per_vertex()
                .definition(&vs.info().input_interface)
                .unwrap();
            let stages = [
                PipelineShaderStageCreateInfo::new(vs),
                PipelineShaderStageCreateInfo::new(fs),
            ];
            let layout = PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                    .into_pipeline_layout_create_info(device.clone())
                    .unwrap(),
            )
            .unwrap();

            GraphicsPipeline::new(
                device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(Default::default()),
                    // TODO: .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                    viewport_state: Some(Default::default()),
                    rasterization_state: Some(Default::default()),
                    multisample_state: Some(Default::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(),
                        Default::default(),
                    )),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(subpass.clone().into()),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap()
        };
        PixelsDrawPipeline {
            gfx_queue,
            subpass,
            pipeline,
            command_buffer_allocator,
            descriptor_set_allocator,
            vertices: vertex_buffer,
            indices: index_buffer,
        }
    }

    fn create_descriptor_set(&self, image: Arc<ImageView>) -> Arc<PersistentDescriptorSet> {
        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let sampler = Sampler::new(
            self.gfx_queue.device().clone(),
            SamplerCreateInfo::simple_repeat_linear(),
        )
        .unwrap();

        PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::image_view_sampler(
                0,
                image.clone(),
                sampler,
            )],
            [],
        )
        .unwrap()
    }

    /// Draw input `image` over a quad of size -1.0 to 1.0
    pub fn draw(
        &mut self,
        viewport_dimensions: [u32; 2],
        image: Arc<ImageView>,
    ) -> Arc<SecondaryAutoCommandBuffer> {
        let mut builder = AutoCommandBufferBuilder::secondary(
            self.command_buffer_allocator.as_ref(),
            self.gfx_queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(self.subpass.clone().into()),
                ..Default::default()
            },
        )
        .unwrap();
        let desc_set = self.create_descriptor_set(image);
        builder
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: [viewport_dimensions[0] as f32, viewport_dimensions[1] as f32],
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                desc_set,
            )
            .unwrap()
            .bind_vertex_buffers(0, self.vertices.clone())
            .unwrap()
            .bind_index_buffer(self.indices.clone())
            .unwrap()
            .draw_indexed(self.indices.len() as u32, 1, 0, 0, 0)
            .unwrap();
        builder.build().unwrap()
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450
layout(location=0) in vec2 position;
layout(location=1) in vec2 tex_coords;

layout(location = 0) out vec2 f_tex_coords;

void main() {
    gl_Position =  vec4(position, 0.0, 1.0);
    f_tex_coords = tex_coords;
}
        "
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout(location = 0) in vec2 v_tex_coords;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
    f_color = texture(tex, v_tex_coords);
}
"
    }
}
