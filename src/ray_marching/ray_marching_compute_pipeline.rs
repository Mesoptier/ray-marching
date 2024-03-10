use std::sync::Arc;

use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Queue;
use vulkano::image::ImageAccess;
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sync::GpuFuture;
use vulkano_util::renderer::DeviceImageView;

use crate::ray_marching::csg::builder::{CSGCommandBufferBuilder, CSGCommandDescriptor};
use crate::ray_marching::csg::operations::subtraction::Subtraction;
use crate::ray_marching::csg::primitives::sphere::Sphere;
use crate::ray_marching::csg::CSGNode;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/ray_marching/ray_marching.glsl",
        types_meta: {
            // TODO: Remove this once vulkano-shaders adds `BufferContents` automatically
            use bytemuck::{Pod, Zeroable};
            #[derive(Copy, Clone, Pod, Zeroable)]
        }
    }
}

pub struct RayMarchingComputePipeline {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ComputePipeline>,
    csg_commands_buffer_pool: CpuBufferPool<CSGCommandDescriptor>,
    csg_params_buffer_pool: CpuBufferPool<u32>,
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

        let csg_commands_buffer_pool = CpuBufferPool::new(
            gfx_queue.device().clone(),
            BufferUsage {
                storage_buffer: true,
                ..BufferUsage::empty()
            },
        );
        let csg_params_buffer_pool = CpuBufferPool::new(
            gfx_queue.device().clone(),
            BufferUsage {
                storage_buffer: true,
                ..BufferUsage::empty()
            },
        );

        Self {
            gfx_queue,
            pipeline,
            csg_commands_buffer_pool,
            csg_params_buffer_pool,
        }
    }

    pub fn compute(&mut self, image: DeviceImageView, t: f32) -> Box<dyn GpuFuture> {
        let dimensions = image.image().dimensions().width_height();

        // Fill CSG buffers
        let node = Subtraction {
            p1: Box::new(Sphere {
                radius: 1.0,
                center: [0.0, 0.0, 0.0],
            }),
            p2: Box::new(Sphere {
                radius: 1.0,
                center: [(t / 20.0).sin(), -(t / 20.0).sin(), (t / 20.0).cos()],
            }),
        };
        let mut builder = CSGCommandBufferBuilder::new();
        node.build_commands(&mut builder);

        let cmd_count = builder.commands.len() as u32;

        let csg_commands_buffer = self
            .csg_commands_buffer_pool
            .from_iter(builder.commands)
            .unwrap();
        let csg_params_buffer = self
            .csg_params_buffer_pool
            .from_iter(builder.params)
            .unwrap();

        // Describe layout
        let pipeline_layout = self.pipeline.layout();
        let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            desc_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, image.clone()),
                WriteDescriptorSet::buffer(1, csg_commands_buffer.clone()),
                WriteDescriptorSet::buffer(2, csg_params_buffer.clone()),
            ],
        )
        .unwrap();

        let push_constants = cs::ty::PushConstants {
            min_dist: 0.001f32,
            max_dist: 100f32,
            cmd_count,
            t,
        };

        // Build primary command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            self.gfx_queue.device().clone(),
            self.gfx_queue.queue_family_index(),
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
