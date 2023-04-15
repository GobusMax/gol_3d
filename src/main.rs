use gol_3d::State;

use std::{collections::VecDeque, time::Instant};

use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const MOVING_AVERAGE_NUM: usize = 10;
fn main() {
    let mut timer = Instant::now();
    let mut moving_average = VecDeque::from([0.; MOVING_AVERAGE_NUM]);

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
        .set_title(&format!("Rule: {}", state.gol.rule));

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
        Event::MainEventsCleared => {
            let delta = timer.elapsed().as_secs_f32();
            moving_average.pop_front();
            moving_average.push_back(delta);
            let _res =
                moving_average.iter().sum::<f32>() / MOVING_AVERAGE_NUM as f32;
            timer = Instant::now();
            state.update(delta);
            // println!("{}", 1. / _res);
            // println!("{}", state.gol.rule)
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
