use std::sync::Arc;

use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Queue;
use vulkano::image::ImageAccess;
use vulkano::memory::allocator::{MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sync::GpuFuture;
use vulkano::DeviceSize;
use vulkano_util::renderer::DeviceImageView;

use crate::ray_marching::csg::builder::CSGCommandBufferBuilder;
use crate::ray_marching::csg::operations::subtraction::Subtraction;
use crate::ray_marching::csg::primitives::sphere::Sphere;
use crate::ray_marching::csg::CSGNode;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/ray_marching/ray_marching.glsl",
    }
}

pub struct RayMarchingComputePipeline {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ComputePipeline>,
    subbuffer_allocator: SubbufferAllocator,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl RayMarchingComputePipeline {
    pub fn new(
        gfx_queue: Arc<Queue>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> Self {
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

        let subbuffer_allocator = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::STORAGE_BUFFER,
                memory_usage: MemoryUsage::Upload,
                ..Default::default()
            },
        );

        Self {
            gfx_queue,
            pipeline,
            subbuffer_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
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
            .subbuffer_allocator
            .allocate_slice(builder.commands.len() as DeviceSize)
            .unwrap();
        csg_commands_buffer
            .write()
            .unwrap()
            .copy_from_slice(&builder.commands);

        let csg_params_buffer = self
            .subbuffer_allocator
            .allocate_slice(builder.params.len() as DeviceSize)
            .unwrap();
        csg_params_buffer
            .write()
            .unwrap()
            .copy_from_slice(&builder.params);

        // Describe layout
        let pipeline_layout = self.pipeline.layout();
        let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            desc_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, image.clone()),
                WriteDescriptorSet::buffer(1, csg_commands_buffer.clone()),
                WriteDescriptorSet::buffer(2, csg_params_buffer.clone()),
            ],
        )
        .unwrap();

        let push_constants = cs::PushConstants {
            min_dist: 0.001f32,
            max_dist: 100f32,
            cmd_count,
            t,
        };

        // Build primary command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
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
