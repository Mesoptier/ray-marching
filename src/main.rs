use std::time::{Duration, Instant};

use winit::event_loop::EventLoop;

use crate::ray_marching::ray_marching_compute_pipeline::RayMarchingComputePipeline;
use crate::renderer::Renderer;

mod renderer;
mod ray_marching;

fn main() {
    let event_loop = EventLoop::new();
    let mut renderer = Renderer::new(&event_loop);

    event_loop.run(move |event, _, control_flow| {
        use winit::event::*;
        use winit::event_loop::ControlFlow;

        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                renderer.render();
            }
            _ => {}
        }
    });
}
