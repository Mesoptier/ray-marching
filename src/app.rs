use crate::ray_marching::ray_marching_compute_pipeline::RayMarchingComputePipeline;
use crate::renderer::render_pass_place_over_frame::RenderPassPlaceOverFrame;
use std::sync::Arc;
use vulkano::device::Queue;
use vulkano::format::Format;

pub(crate) struct App {
    pub render_pass: RenderPassPlaceOverFrame,
    pub compute_pipeline: RayMarchingComputePipeline,
}

impl App {
    pub fn new(gfx_queue: Arc<Queue>, image_format: Format) -> Self {
        let render_pass = RenderPassPlaceOverFrame::new(gfx_queue.clone(), image_format);
        let compute_pipeline = RayMarchingComputePipeline::new(gfx_queue);

        Self {
            render_pass,
            compute_pipeline,
        }
    }
}
