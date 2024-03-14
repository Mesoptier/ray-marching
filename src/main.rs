use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::mutex::Mutex;
use egui::Frame;
use egui_winit_vulkano::{CallbackFn, Gui};
use vulkano::sync::GpuFuture;
use vulkano_util::context::{VulkanoConfig, VulkanoContext};
use vulkano_util::window::{VulkanoWindows, WindowDescriptor};
use winit::event_loop::EventLoop;

use crate::scene::Scene;

mod gui;
mod ray_marching;
mod renderer;
mod scene;

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

    let mut node_graph = gui::Gui::default();

    let mut gui = Gui::new(
        &event_loop,
        primary_window_renderer.surface(),
        primary_window_renderer.graphics_queue(),
        primary_window_renderer.swapchain_format(),
        Default::default(),
    );

    let scene = Arc::new(Mutex::new(Scene::new(gui.render_resources())));

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
                            node_graph.draw(ui);
                        });

                    egui::CentralPanel::default().show(&ctx, |ui| {
                        Frame::canvas(ui.style()).show(ui, |ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());

                            scene.lock().update_extent(
                                rect.width().round() as u32,
                                rect.height().round() as u32,
                            );

                            let scene = scene.clone();
                            let paint_callback = egui::PaintCallback {
                                rect,
                                callback: Arc::new(CallbackFn::new(move |info, ctx| {
                                    scene.lock().render(info, ctx);
                                })),
                            };

                            ui.painter().add(paint_callback);
                        });
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

                // Compute scene
                let render_scene_future = scene
                    .lock()
                    .compute(
                        start_time.elapsed().as_secs_f32(),
                        node_graph.evaluate_root(),
                    )
                    .join(before_pipeline_future);

                // Render GUI
                let final_image = primary_window_renderer.swapchain_image_view();
                let after_render_pass_future = gui.draw_on_image(render_scene_future, final_image);

                // Finish frame
                primary_window_renderer.present(after_render_pass_future, true);
            }
            _ => {}
        }
    });
}
