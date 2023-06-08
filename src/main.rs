#![feature(async_closure)]

use gol_3d::State;

use std::collections::VecDeque;

use instant::Instant;

use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const MOVING_AVERAGE_NUM: usize = 10;

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut timer = Instant::now();
    let mut moving_average = VecDeque::from([0.; MOVING_AVERAGE_NUM]);

    // log::log!(log::Level::Info, "0");

    let mut state = State::new(window).await;

    #[cfg(not(target_arch = "wasm32"))]
    state
        .env
        .window
        .set_title(&format!("Rule: {}", state.gol.rule));

    // log::log!(log::Level::Info, "1");

    let f = async move |event: Event<()>, _, control_flow: &mut ControlFlow| {
        // log::log!(log::Level::Info, "2");

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.env.window.id() => {
                // log::log!(log::Level::Info, "y1");
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                winit::event::KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode:
                                        Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,

                        WindowEvent::Resized(physicalsize) => {
                            state.resize(*physicalsize);
                        }
                        WindowEvent::ScaleFactorChanged {
                            new_inner_size,
                            ..
                        } => {
                            state.resize(**new_inner_size);
                        }

                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                // log::log!(log::Level::Info, "y2");
                let delta = timer.elapsed().as_secs_f32();
                moving_average.pop_front();
                moving_average.push_back(delta);
                let _res = moving_average.iter().sum::<f32>()
                    / MOVING_AVERAGE_NUM as f32;
                timer = Instant::now();
                state.update(delta).await;
                // println!("{}", 1. / _res);
                // println!("{}", state.gol.rule)
            }
            Event::DeviceEvent {
                device_id: _,
                event,
            } => {
                // log::log!(log::Level::Info, "y3");
                state.camera.controller.process_mouse(&event);
            }
            _ => {}
        }
    };

    event_loop.run(f);
}

fn main() {
    let event_loop = EventLoop::new();

    // #[cfg(not(target_arch = "wasm32"))]
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(900, 900))
        .build(&event_loop)
        .unwrap();

    // #[cfg(target_arch = "wasm32")]
    // let window = Window::new(&event_loop).unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run(event_loop, window));
    }

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
