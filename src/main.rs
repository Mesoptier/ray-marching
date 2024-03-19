use eframe::{egui, egui_wgpu};

use crate::ray_marching::renderer::{RayMarchingCallback, RayMarchingResources};

mod camera;
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
}

struct RayMarchingApp {
    csg_node_graph: csg_node_graph::CSGNodeGraph,
    camera_controller: camera::CameraController,
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
            camera_controller: camera::CameraController::new([0.0, 0.0, 0.0], 5.0),
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
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

                if response.dragged_by(egui::PointerButton::Primary) {
                    let modifiers = ctx.input(|input_state| input_state.modifiers);
                    let delta = response.drag_delta().into();
                    if modifiers.ctrl {
                        self.camera_controller
                            .update(camera::CameraControllerEvent::Pan(delta));
                    } else {
                        self.camera_controller
                            .update(camera::CameraControllerEvent::Orbit(delta));
                    }
                }

                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    rect,
                    RayMarchingCallback::new(
                        0.0,
                        self.csg_node_graph.evaluate_root(),
                        [rect.width(), rect.height()],
                        self.camera_controller.camera(),
                    ),
                ));
            });
        });
    }
}
