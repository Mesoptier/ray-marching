use egui::Frame;
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

    let scene_image_view_id = 0;
    primary_window_renderer.add_additional_image_view(
        scene_image_view_id,
        DEFAULT_IMAGE_FORMAT,
        ImageUsage::SAMPLED | ImageUsage::INPUT_ATTACHMENT | ImageUsage::STORAGE,
    );

    let mut app = App::new(
        context.graphics_queue().clone(),
        primary_window_renderer.swapchain_format(),
    );

    let mut gui = Gui::new(
        &event_loop,
        primary_window_renderer.surface(),
        primary_window_renderer.graphics_queue(),
        primary_window_renderer.swapchain_format(),
        Default::default(),
    );

    let scene_image_view = primary_window_renderer.get_additional_image_view(scene_image_view_id);
    let scene_texture_id =
        gui.register_user_image_view(scene_image_view.clone(), Default::default());

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
            Event::WindowEvent { event, .. } => {
                gui.update(&event);

                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(..) => primary_window_renderer.resize(),
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                gui.immediate_ui(|gui| {
                    let ctx = gui.context();

                    egui::TopBottomPanel::bottom("node_graph")
                        .resizable(true)
                        .show(&ctx, |ui| {
                            app.gui.draw(ui);
                        });

                    egui::CentralPanel::default()
                        .frame(Frame::default().inner_margin(0.))
                        .show(&ctx, |ui| {
                            ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                                scene_texture_id,
                                [ui.available_width(), ui.available_height()],
                            )));
                        });
                });

                // Start frame
                let before_pipeline_future = match primary_window_renderer.acquire() {
                    Err(e) => {
                        println!("{}", e);
                        return;
                    }
                    Ok(future) => future,
                };

                // Compute & render
                let final_image = primary_window_renderer.swapchain_image_view();

                let render_scene_future = app
                    .compute(scene_image_view.clone(), start_time.elapsed().as_secs_f32())
                    .join(before_pipeline_future);

                // Render GUI
                let after_render_pass_future = gui.draw_on_image(render_scene_future, final_image);

                // Finish frame
                primary_window_renderer.present(after_render_pass_future, true);
            }
            _ => {}
        }
    });
}
