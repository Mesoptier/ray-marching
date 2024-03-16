use eframe::{egui, egui_wgpu};

use crate::ray_marching::renderer::{RayMarchingCallback, RayMarchingResources};

mod csg_node_graph;
mod ray_marching;

fn main() {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Ray Marching Demo",
        native_options,
        Box::new(|ctx| Box::new(RayMarchingApp::new(ctx))),
    )
    .unwrap()

    //     let context = VulkanoContext::new(VulkanoConfig::default());
    //     let event_loop = EventLoop::new();
    //
    //     let mut windows = VulkanoWindows::default();
    //     let _primary_window_id = windows.create_window(
    //         &event_loop,
    //         &context,
    //         &WindowDescriptor {
    //             title: "Ray Marching Demo".into(),
    //             ..Default::default()
    //         },
    //         |_| {},
    //     );
    //
    //     let primary_window_renderer = windows.get_primary_renderer_mut().unwrap();
    //
    //     let mut node_graph = gui::Gui::default();
    //
    //     let mut gui = Gui::new(
    //         &event_loop,
    //         primary_window_renderer.surface(),
    //         primary_window_renderer.graphics_queue(),
    //         primary_window_renderer.swapchain_format(),
    //         Default::default(),
    //     );
    //
    //     let scene = Arc::new(Mutex::new(Scene::new(gui.render_resources())));
    //
    //     let start_time = Instant::now();
    //
    //     event_loop.run(move |event, _, control_flow| {
    //         use winit::event::*;
    //         use winit::event_loop::ControlFlow;
    //
    //         let primary_window_renderer = windows.get_primary_renderer_mut().unwrap();
    //
    //         match event {
    //             Event::NewEvents(_sc) => {
    //                 let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
    //                 *control_flow = ControlFlow::WaitUntil(next_frame_time);
    //             }
    //             Event::WindowEvent { event, .. } => {
    //                 gui.update(&event);
    //
    //                 match event {
    //                     WindowEvent::CloseRequested => {
    //                         *control_flow = ControlFlow::Exit;
    //                     }
    //                     WindowEvent::Resized(..) => primary_window_renderer.resize(),
    //                     _ => {}
    //                 }
    //             }
    //             Event::MainEventsCleared => {
    //                 gui.immediate_ui(|gui| {
    //                     let ctx = gui.context();
    //
    //                     egui::TopBottomPanel::bottom("node_graph")
    //                         .resizable(true)
    //                         .show(&ctx, |ui| {
    //                             node_graph.draw(ui);
    //                         });
    //
    //                     egui::CentralPanel::default().show(&ctx, |ui| {
    //                         Frame::canvas(ui.style()).show(ui, |ui| {
    //                             let (rect, _) =
    //                                 ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
    //
    //                             scene.lock().update_extent(
    //                                 rect.width().round() as u32,
    //                                 rect.height().round() as u32,
    //                             );
    //
    //                             let scene = scene.clone();
    //                             let paint_callback = egui::PaintCallback {
    //                                 rect,
    //                                 callback: Arc::new(CallbackFn::new(move |info, ctx| {
    //                                     scene.lock().render(info, ctx);
    //                                 })),
    //                             };
    //
    //                             ui.painter().add(paint_callback);
    //                         });
    //                     });
    //                 });
    //
    //                 // Start frame
    //                 let before_pipeline_future = match primary_window_renderer.acquire() {
    //                     Err(e) => {
    //                         println!("{}", e);
    //                         return;
    //                     }
    //                     Ok(future) => future,
    //                 };
    //
    //                 // Compute scene
    //                 let render_scene_future = scene
    //                     .lock()
    //                     .compute(
    //                         start_time.elapsed().as_secs_f32(),
    //                         node_graph.evaluate_root(),
    //                     )
    //                     .join(before_pipeline_future);
    //
    //                 // Render GUI
    //                 let final_image = primary_window_renderer.swapchain_image_view();
    //                 let after_render_pass_future = gui.draw_on_image(render_scene_future, final_image);
    //
    //                 // Finish frame
    //                 primary_window_renderer.present(after_render_pass_future, true);
    //             }
    //             _ => {}
    //         }
    //     });
}

struct RayMarchingApp {
    csg_node_graph: csg_node_graph::CSGNodeGraph,
}

impl RayMarchingApp {
    fn new(ctx: &eframe::CreationContext) -> Self {
        let wgpu_render_state = ctx.wgpu_render_state.as_ref().unwrap();
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(RayMarchingResources::new(wgpu_render_state));

        Self {
            csg_node_graph: csg_node_graph::CSGNodeGraph::default(),
        }
    }
}

impl eframe::App for RayMarchingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("node_graph")
            .resizable(true)
            .show(ctx, |ui| {
                self.csg_node_graph.draw(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    rect,
                    RayMarchingCallback::new(
                        0.0,
                        self.csg_node_graph.evaluate_root(),
                        [rect.width(), rect.height()],
                    ),
                ));
            });
        });
    }
}
