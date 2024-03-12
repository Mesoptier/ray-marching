use std::time::{Duration, Instant};

use egui_winit_vulkano::Gui;
use vulkano::image::ImageUsage;
use vulkano::sync::GpuFuture;
use vulkano_util::context::{VulkanoConfig, VulkanoContext};
use vulkano_util::renderer::DEFAULT_IMAGE_FORMAT;
use vulkano_util::window::{VulkanoWindows, WindowDescriptor};
use winit::event_loop::EventLoop;

use crate::app::App;

mod app;
mod gui;
mod ray_marching;
mod renderer;

fn main() {
    let context = VulkanoContext::new(VulkanoConfig::default());
    let event_loop = EventLoop::new();

    let mut windows = VulkanoWindows::default();
    let primary_window_id = windows.create_window(
        &event_loop,
        &context,
        &WindowDescriptor {
            title: "Ray Marching Demo".into(),
            ..Default::default()
        },
        |_| {},
    );
    let gui_window_id = windows.create_window(
        &event_loop,
        &context,
        &WindowDescriptor {
            title: "GUI".into(),
            ..Default::default()
        },
        |_| {},
    );

    let primary_window_renderer = windows.get_primary_renderer_mut().unwrap();

    let render_target_id = 0;
    primary_window_renderer.add_additional_image_view(
        render_target_id,
        DEFAULT_IMAGE_FORMAT,
        ImageUsage::SAMPLED | ImageUsage::INPUT_ATTACHMENT | ImageUsage::STORAGE,
    );

    let mut app = App::new(
        context.graphics_queue().clone(),
        primary_window_renderer.swapchain_format(),
    );

    let gui_window_renderer = windows.get_renderer(gui_window_id).unwrap();
    let mut gui = Gui::new(
        &event_loop,
        gui_window_renderer.surface(),
        gui_window_renderer.graphics_queue(),
        gui_window_renderer.swapchain_format(),
        Default::default(),
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
            Event::WindowEvent { event, window_id } => {
                if window_id == gui_window_id {
                    gui.update(&event);
                }
                if window_id == primary_window_id {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(..) => primary_window_renderer.resize(),
                        _ => {}
                    }
                }
            }
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
                    .compute(render_target.clone(), start_time.elapsed().as_secs_f32())
                    .join(before_pipeline_future);

                let after_render_pass_future = app.render_pass.render(
                    compute_future,
                    render_target,
                    primary_window_renderer.swapchain_image_view(),
                );

                // Finish frame
                primary_window_renderer.present(after_render_pass_future, true);

                // TODO: Should this be called every MainEventsCleared?
                let gui_window_renderer = windows.get_renderer(gui_window_id).unwrap();
                gui_window_renderer.window().request_redraw();
            }
            Event::RedrawRequested(window_id) => {
                if window_id == gui_window_id {
                    gui.immediate_ui(|gui| {
                        let ctx = gui.context();

                        egui::CentralPanel::default().show(&ctx, |ui| {
                            app.gui.draw(ui);
                        });
                    });

                    let gui_window_renderer = windows.get_renderer_mut(gui_window_id).unwrap();
                    let before_future = gui_window_renderer.acquire().unwrap();
                    let after_future = gui
                        .draw_on_image(before_future, gui_window_renderer.swapchain_image_view());
                    gui_window_renderer.present(after_future, true);
                }
            }
            _ => {}
        }
    });
}
