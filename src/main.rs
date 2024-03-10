use std::time::{Duration, Instant};

use vulkano::image::ImageUsage;
use vulkano::sync::GpuFuture;
use vulkano_util::context::{VulkanoConfig, VulkanoContext};
use vulkano_util::renderer::DEFAULT_IMAGE_FORMAT;
use vulkano_util::window::{VulkanoWindows, WindowDescriptor};
use winit::event_loop::EventLoop;

use crate::app::App;

mod app;
mod ray_marching;
mod renderer;

fn main() {
    let context = VulkanoContext::new(VulkanoConfig::default());
    let event_loop = EventLoop::new();

    let mut windows = VulkanoWindows::default();
    let _primary_window_id = windows.create_window(
        &event_loop,
        &context,
        &WindowDescriptor {
            title: "Ray Marching Demo".into(),
            ..Default::default()
        },
        |_| {},
    );

    let primary_window_renderer = windows.get_primary_renderer_mut().unwrap();

    let render_target_id = 0;
    primary_window_renderer.add_additional_image_view(
        render_target_id,
        DEFAULT_IMAGE_FORMAT,
        ImageUsage {
            sampled: true,
            input_attachment: true,
            storage: true,
            ..ImageUsage::empty()
        },
    );

    let mut app = App::new(
        context.graphics_queue().clone(),
        primary_window_renderer.swapchain_format(),
    );

    let start_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        use winit::event::*;
        use winit::event_loop::ControlFlow;

        let primary_window_renderer = windows.get_primary_renderer_mut().unwrap();

        match event {
            Event::NewEvents(_sc) => {
                let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(..),
                ..
            } => primary_window_renderer.resize(),
            Event::MainEventsCleared => {
                // Start frame
                let before_pipeline_future = match primary_window_renderer.acquire() {
                    Err(e) => {
                        println!("{}", e);
                        return;
                    }
                    Ok(future) => future,
                };

                // Compute & render
                let render_target =
                    primary_window_renderer.get_additional_image_view(render_target_id);

                let compute_future = app
                    .compute_pipeline
                    .compute(render_target.clone(), start_time.elapsed().as_secs_f32())
                    .join(before_pipeline_future);

                let after_render_pass_future = app.render_pass.render(
                    compute_future,
                    render_target,
                    primary_window_renderer.swapchain_image_view(),
                );

                // Finish frame
                primary_window_renderer.present(after_render_pass_future, true);
            }
            _ => {}
        }
    });
}
