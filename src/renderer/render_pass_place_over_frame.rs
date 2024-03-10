// Copyright (c) 2021 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::sync::Arc;

use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{RenderPassBeginInfo, SubpassBeginInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::MemoryAllocator;
use vulkano::render_pass::FramebufferCreateInfo;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents},
    device::Queue,
    format::Format,
    render_pass::{Framebuffer, RenderPass, Subpass},
    sync::GpuFuture,
};

use crate::renderer::pixels_draw_pipeline::PixelsDrawPipeline;

/// A render pass which places an incoming image over frame filling it
pub struct RenderPassPlaceOverFrame {
    gfx_queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pixels_draw_pipeline: PixelsDrawPipeline,
}

impl RenderPassPlaceOverFrame {
    pub fn new(
        gfx_queue: Arc<Queue>,
        memory_allocator: Arc<dyn MemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
        output_format: Format,
    ) -> RenderPassPlaceOverFrame {
        let render_pass = vulkano::single_pass_renderpass!(gfx_queue.device().clone(),
            attachments: {
                color: {
                    format: output_format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap();
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        let pixels_draw_pipeline = PixelsDrawPipeline::new(
            gfx_queue.clone(),
            subpass,
            memory_allocator,
            command_buffer_allocator.clone(),
            descriptor_set_allocator,
        );
        RenderPassPlaceOverFrame {
            gfx_queue,
            render_pass,
            command_buffer_allocator,
            pixels_draw_pipeline,
        }
    }

    /// Place view exactly over swapchain image target.
    /// Texture draw pipeline uses a quad onto which it places the view.
    pub fn render<F>(
        &mut self,
        before_future: F,
        view: Arc<ImageView>,
        target: Arc<ImageView>,
    ) -> Box<dyn GpuFuture>
    where
        F: GpuFuture + 'static,
    {
        // Get dimensions
        let [width, height, _] = target.image().extent();

        // Create framebuffer (must be in same order as render pass description in `new`
        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![target],
                ..Default::default()
            },
        )
        .unwrap();

        // Create primary command buffer builder
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.gfx_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Begin render pass
        command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0; 4].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassBeginInfo {
                    contents: SubpassContents::SecondaryCommandBuffers,
                    ..Default::default()
                },
            )
            .unwrap();

        // Create secondary command buffer from texture pipeline & send draw commands
        let cb = self.pixels_draw_pipeline.draw([width, height], view);

        // Execute above commands (subpass)
        command_buffer_builder.execute_commands(cb).unwrap();

        // End render pass
        command_buffer_builder
            .end_render_pass(Default::default())
            .unwrap();

        // Build command buffer
        let command_buffer = command_buffer_builder.build().unwrap();

        // Execute primary command buffer
        let after_future = before_future
            .then_execute(self.gfx_queue.clone(), command_buffer)
            .unwrap();

        after_future.boxed()
    }
}
