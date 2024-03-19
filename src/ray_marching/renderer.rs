use eframe::egui::PaintCallbackInfo;
use eframe::egui_wgpu::{CallbackResources, CallbackTrait, RenderState};
use wgpu::util::DeviceExt;
use wgpu::{
    CommandBuffer, CommandEncoder, Device, PrimitiveState, PrimitiveTopology, Queue, RenderPass,
};

use crate::ray_marching::csg::builder::CSGCommandBufferBuilder;
use crate::ray_marching::csg::{BuildCommands, CSGNode};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RayMarchLimits {
    min_dist: f32,
    max_dist: f32,
    max_iter: u32,
}

pub struct RayMarchingResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,

    cmd_buffer: wgpu::Buffer,
    viewport_buffer: wgpu::Buffer,
}

impl RayMarchingResources {
    pub fn new(wgpu_render_state: &RenderState) -> Self {
        let device = &wgpu_render_state.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ray_marching"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./ray_marching.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ray_marching"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ray_marching"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ray_marching"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu_render_state.target_format.into())],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });

        let viewport_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("viewport"),
            contents: bytemuck::cast_slice(&[512.0, 512.0]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let ray_march_limits_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ray_march_limits"),
                contents: bytemuck::cast_slice(&[RayMarchLimits {
                    min_dist: 0.01,
                    max_dist: 100.0,
                    max_iter: 100,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let cmd_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ray_marching_cmd_buffer"),
            size: 1024,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ray_marching"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ray_march_limits_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: cmd_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: viewport_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            cmd_buffer,
            viewport_buffer,
        }
    }
}

pub struct RayMarchingCallback {
    time: f32,
    csg_node: Option<CSGNode>,
    viewport: [f32; 2],
}

impl RayMarchingCallback {
    pub fn new(time: f32, csg_node: Option<CSGNode>, viewport: [f32; 2]) -> Self {
        Self {
            time,
            csg_node,
            viewport,
        }
    }
}

impl CallbackTrait for RayMarchingCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        egui_encoder: &mut CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let resources: &RayMarchingResources = callback_resources.get().unwrap();

        queue.write_buffer(
            &resources.viewport_buffer,
            0,
            bytemuck::cast_slice(&self.viewport),
        );

        let mut builder = CSGCommandBufferBuilder::new();
        if let Some(csg_node) = &self.csg_node {
            csg_node.build_commands(&mut builder);
        }

        // TODO: Recreate the buffers if they are too small
        queue.write_buffer(
            &resources.cmd_buffer,
            0,
            bytemuck::cast_slice(&[builder.cmd_count]),
        );
        queue.write_buffer(
            &resources.cmd_buffer,
            4,
            bytemuck::cast_slice(&builder.buffer),
        );

        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'a>,
        callback_resources: &'a CallbackResources,
    ) {
        let resources: &RayMarchingResources = callback_resources.get().unwrap();

        render_pass.set_pipeline(&resources.pipeline);
        render_pass.set_bind_group(0, &resources.bind_group, &[]);
        render_pass.draw(0..4, 0..2);
    }
}
