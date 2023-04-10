use gol_3d::State;

use std::time::Instant;

use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
fn main() {
    let mut timer = Instant::now();
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(900, 900))
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(window);
    state
        .env
        .window
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();
    state.env.window.set_cursor_visible(false);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.env.window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,

                    WindowEvent::Resized(physicalsize) => {
                        state.resize(*physicalsize);
                    }
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size, ..
                    } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id)
            if window_id == state.env.window.id() =>
        {
            let delta = timer.elapsed().as_secs_f32();
            timer = Instant::now();
            state.update(delta, control_flow);
            // println!(
            //     "{}",
            //     1. / delta
            // )
        }
        Event::MainEventsCleared => {
            state.env.window.request_redraw();
        }
        Event::DeviceEvent {
            device_id: _,
            event,
        } => {
            state.camera.controller.process_mouse(&event);
        }
        _ => {}
    });
}
