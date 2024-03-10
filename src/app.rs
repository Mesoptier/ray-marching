use std::sync::Arc;

use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::memory::allocator::StandardMemoryAllocator;

use crate::ray_marching::ray_marching_compute_pipeline::RayMarchingComputePipeline;
use crate::renderer::render_pass_place_over_frame::RenderPassPlaceOverFrame;

pub(crate) struct App {
    pub render_pass: RenderPassPlaceOverFrame,
    pub compute_pipeline: RayMarchingComputePipeline,
}

impl App {
    pub fn new(gfx_queue: Arc<Queue>, image_format: Format) -> Self {
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(
            gfx_queue.device().clone(),
        ));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            gfx_queue.device().clone(),
            Default::default(),
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            gfx_queue.device().clone(),
        ));

        Self {
            render_pass: RenderPassPlaceOverFrame::new(
                gfx_queue.clone(),
                &memory_allocator,
                command_buffer_allocator.clone(),
                descriptor_set_allocator.clone(),
                image_format,
            ),
            compute_pipeline: RayMarchingComputePipeline::new(
                gfx_queue,
                memory_allocator,
                command_buffer_allocator,
                descriptor_set_allocator,
            ),
        }
    }
}
