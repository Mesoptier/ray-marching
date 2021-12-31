use std::time::{Duration, Instant};

use winit::event_loop::EventLoop;

use crate::ray_marching::ray_marching_compute_pipeline::RayMarchingComputePipeline;
use crate::renderer::Renderer;

mod renderer;
mod ray_marching;

fn main() {
    let event_loop = EventLoop::new();
    let mut renderer = Renderer::new(&event_loop);

    let start_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        use winit::event::*;
        use winit::event_loop::ControlFlow;

        match event {
            Event::NewEvents(sc) => {
                let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::Resized(..), .. } => {
                renderer.resize();
            }
            Event::MainEventsCleared => {
                let running_time = Instant::now() - start_time;
                renderer.render(running_time.as_secs_f32());
            }
            _ => {}
        }
    });
}
