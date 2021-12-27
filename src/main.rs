use winit::event_loop::EventLoop;

use crate::renderer::Renderer;

mod renderer;

fn main() {
    let event_loop = EventLoop::new();
    let renderer = Renderer::new(&event_loop);

    event_loop.run(|event, _, control_flow| {
        use winit::event::*;
        use winit::event_loop::ControlFlow;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
