use eframe::egui::PaintCallbackInfo;
use eframe::egui_wgpu::{CallbackResources, CallbackTrait, RenderState};
use wgpu::RenderPass;

use crate::ray_marching::csg::CSGNode;

pub struct RayMarchingResources {
    pipeline: wgpu::RenderPipeline,
}

impl RayMarchingResources {
    pub fn new(wgpu_render_state: &RenderState) -> Self {
        let device = &wgpu_render_state.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ray_marching"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./ray_marching.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ray_marching"),
            bind_group_layouts: &[],
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
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });

        Self { pipeline }
    }
}

pub struct RayMarchingCallback {
    time: f32,
    csg_node: Option<CSGNode>,
}

impl RayMarchingCallback {
    pub fn new(time: f32, csg_node: Option<CSGNode>) -> Self {
        Self { time, csg_node }
    }
}

impl CallbackTrait for RayMarchingCallback {
    fn paint<'a>(
        &'a self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'a>,
        resources: &'a CallbackResources,
    ) {
        let resources: &RayMarchingResources = resources.get().unwrap();

        render_pass.set_pipeline(&resources.pipeline);
        render_pass.draw(0..3, 0..1);
    }
}
