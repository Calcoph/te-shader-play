use winit::{event::{Event, WindowEvent}, event_loop::{EventLoopWindowTarget, ControlFlow}, window::Window};

use crate::{State, rendering::render};


pub fn run_event_loop(event: Event<()>, window_target: &EventLoopWindowTarget<()>, window: &Window, state: &mut State) {
    window_target.set_control_flow(ControlFlow::Poll);
    match event {
        Event::WindowEvent { window_id: _, event } => handle_window_event(event, window_target, state),
        Event::Suspended => window_target.set_control_flow(ControlFlow::Wait),
        Event::AboutToWait => window.request_redraw(),
        _ => ()
    }
}

fn handle_window_event(event: WindowEvent, window_target: &EventLoopWindowTarget<()>, state: &mut State) {
    match event {
        WindowEvent::CloseRequested => window_target.exit(),
        WindowEvent::RedrawRequested => {
            let dt = state.time.update_time(&state.gpu.queue);
            if let Ok(output) = state.gpu.surface.get_current_texture() {
                render(output, state);
            }
        },
        WindowEvent::KeyboardInput { event, .. } => handle_keyboard(event, state),
        _ => ()
    }
}

fn handle_keyboard(event: winit::event::KeyEvent, state: &mut State) {
    match event.physical_key {
        winit::keyboard::PhysicalKey::Code(c) => match c {
            winit::keyboard::KeyCode::KeyQ => state.refresh_default_shader(),
            _ => (),
        },
        winit::keyboard::PhysicalKey::Unidentified(_) => (),
    }
}
