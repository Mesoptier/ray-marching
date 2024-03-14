use std::sync::Arc;

use egui::PaintCallbackInfo;
use egui_winit_vulkano::{CallbackContext, RenderResources};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageUsage};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::sync::GpuFuture;
use vulkano_util::renderer::DEFAULT_IMAGE_FORMAT;

use crate::ray_marching::csg::CSGNode;
use crate::ray_marching::ray_marching_compute_pipeline::RayMarchingComputePipeline;
use crate::renderer::pixels_draw_pipeline::PixelsDrawPipeline;

pub(crate) struct Scene {
    image_view: Arc<ImageView>,
    graphics_pipeline: PixelsDrawPipeline,
    compute_pipeline: RayMarchingComputePipeline,
    memory_allocator: Arc<StandardMemoryAllocator>,
}

impl Scene {
    pub fn new(resources: RenderResources) -> Self {
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            resources.queue.device().clone(),
            StandardCommandBufferAllocatorCreateInfo {
                secondary_buffer_count: 32,
                ..Default::default()
            },
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            resources.queue.device().clone(),
            Default::default(),
        ));

        Self {
            image_view: Self::create_image_view(resources.memory_allocator.clone(), 1, 1),
            graphics_pipeline: PixelsDrawPipeline::new(
                resources.queue.clone(),
                resources.subpass.clone(),
                resources.memory_allocator.clone(),
            ),
            compute_pipeline: RayMarchingComputePipeline::new(
                resources.queue.clone(),
                resources.memory_allocator.clone(),
                command_buffer_allocator,
                descriptor_set_allocator,
            ),
            memory_allocator: resources.memory_allocator.clone(),
        }
    }

    pub fn update_extent(&mut self, width: u32, height: u32) {
        // TODO: Only recreate the image view if the extent has changed
        self.image_view = Self::create_image_view(self.memory_allocator.clone(), width, height);
    }

    fn create_image_view(
        memory_allocator: Arc<StandardMemoryAllocator>,
        width: u32,
        height: u32,
    ) -> Arc<ImageView> {
        ImageView::new_default(
            Image::new(
                memory_allocator,
                ImageCreateInfo {
                    extent: [width, height, 1],
                    usage: ImageUsage::SAMPLED | ImageUsage::INPUT_ATTACHMENT | ImageUsage::STORAGE,
                    format: DEFAULT_IMAGE_FORMAT,
                    ..Default::default()
                },
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    }

    pub fn compute(&mut self, time: f32, csg_node: Option<CSGNode>) -> Box<dyn GpuFuture> {
        self.compute_pipeline
            .compute(self.image_view.clone(), time, csg_node)
    }

    pub fn render(&self, info: PaintCallbackInfo, ctx: &mut CallbackContext) {
        self.graphics_pipeline
            .draw(self.image_view.clone(), info, ctx);
    }
}
