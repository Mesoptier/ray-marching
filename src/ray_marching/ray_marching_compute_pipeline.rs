use std::sync::Arc;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Queue;
use vulkano::image::ImageAccess;
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sync::GpuFuture;

use crate::renderer::InterimImageView;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/ray_marching/ray_marching.glsl"
    }
}

pub struct RayMarchingComputePipeline {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ComputePipeline>,
    scene_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl RayMarchingComputePipeline {
    pub fn new(gfx_queue: Arc<Queue>) -> Self {
        let pipeline = {
            let cs = cs::load(gfx_queue.device().clone()).unwrap();
            ComputePipeline::new(
                gfx_queue.device().clone(),
                cs.entry_point("main").unwrap(),
                &(),
                None,
                |_| {},
            )
            .unwrap()
        };

        let scene_buffer = CpuAccessibleBuffer::from_iter(
            gfx_queue.device().clone(),
            BufferUsage::all(),
            false,
            // Single sphere with radius 1 positioned at (0, 0, 0)
            [0.0f32.to_bits(), 0.0f32.to_bits(), 0.0f32.to_bits(), 1.0f32.to_bits()].into_iter(),
        )
        .unwrap();

        Self {
            gfx_queue,
            pipeline,
            scene_buffer,
        }
    }

    pub fn compute(&mut self, image: InterimImageView) -> Box<dyn GpuFuture> {
        let dimensions = image.image().dimensions().width_height();

        // Describe layout
        let pipeline_layout = self.pipeline.layout();
        let desc_layout = pipeline_layout.descriptor_set_layouts().get(0).unwrap();
        let mut desc_set_builder = PersistentDescriptorSet::start(desc_layout.clone());
        desc_set_builder
            .add_image(image.clone())
            .unwrap()
            .add_buffer(self.scene_buffer.clone())
            .unwrap();
        let set = desc_set_builder.build().unwrap();

        let push_constants = cs::ty::PushConstants {
            min_dist: 0.001f32,
            max_dist: 10f32,
            scene_size: self.scene_buffer.len() as u32,
        };

        // Build primary command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .bind_pipeline_compute(self.pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .dispatch([dimensions[0] / 8, dimensions[1] / 8, 1])
            .unwrap();

        // Build and execute commands
        let command_buffer = builder.build().unwrap();
        let finished = command_buffer.execute(self.gfx_queue.clone()).unwrap();
        finished.then_signal_fence_and_flush().unwrap().boxed()
    }
}
