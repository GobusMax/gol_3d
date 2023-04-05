use gol_3d::State;
use pollster::FutureExt;
use std::time::Instant;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferAddress,
    BufferDescriptor, BufferUsages, CommandEncoderDescriptor,
    ComputePassDescriptor, Device, Queue, ShaderStages,
};
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

    //TEST START
    let test_res = test(&state.env.device, &state.env.queue)
        .block_on()
        .unwrap();
    println!("{:?}", test_res);
    //TEST END

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
            state.update(delta);
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.env.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    *control_flow = ControlFlow::Exit;
                }
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{e:?}"),
            }
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

async fn test(device: &Device, queue: &Queue) -> Option<Vec<u32>> {
    let data: [u32; 4] = [0, 0, 0, 0];
    let size = (data.len() * std::mem::size_of::<u32>()) as BufferAddress;
    let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let storage_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Compute Buffer"),
        contents: bytemuck::cast_slice(&data),
        usage: BufferUsages::STORAGE
            | BufferUsages::COPY_SRC
            | BufferUsages::COPY_DST,
    });

    let bind_group_layout =
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE | ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });
    let compute_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
    let compute_pipeline =
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: "cs_main",
        });

    let mut encoder =
        device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });
    encoder.insert_debug_marker("Test");
    {
        let mut compute_pass =
            encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        compute_pass.dispatch_workgroups(data.len() as u32, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);
    queue.submit(Some(encoder.finish()));
    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) =
        futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice
        .map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
    device.poll(wgpu::Maintain::Wait);
    if let Some(Ok(())) = receiver.receive().await {
        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to u32
        let result = bytemuck::cast_slice(&data).to_vec();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        staging_buffer.unmap(); // Unmaps buffer from memory
                                // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                //   delete myPointer;
                                //   myPointer = NULL;
                                // It effectively frees the memory

        // Returns data from buffer
        Some(result)
    } else {
        panic!("failed to run compute on gpu!")
    }
}
