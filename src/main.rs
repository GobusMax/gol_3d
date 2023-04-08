use gol_3d::State;
use ndarray::Array3;
use pollster::FutureExt;
use std::time::Instant;
use wgpu::{
    include_wgsl, util::DeviceExt, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, CommandEncoderDescriptor, ComputePassDescriptor,
    Device, Extent3d, Queue, ShaderStages, TextureDescriptor, TextureFormat,
    TextureUsages, TextureViewDescriptor,
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
    // test(&state.env.device, &state.env.queue).block_on();
    // match state.render() {
    //     Ok(_) => {}
    //     // Reconfigure the surface if lost
    //     Err(wgpu::SurfaceError::Lost) => state.resize(state.env.size),
    //     // The system is out of memory, we should probably quit
    //     Err(wgpu::SurfaceError::OutOfMemory) => {}
    //     // All other errors (Outdated, Timeout) should be resolved by the next frame
    //     Err(e) => eprintln!("{e:?}"),
    // }
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

async fn test(device: &Device, queue: &Queue) {
    let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
    let l = 100;
    // println!("{:?}", device.limits());
    let (compute_pipeline, bind_group) = {
        let mut cells: Array3<u8> = Array3::<u8>::ones((l, l, l));
        cells.iter_mut().enumerate().for_each(|(i, v)| *v = i as u8);
        let dataflat: Vec<u8> = cells.into_raw_vec();
        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Cells Texture"),
                size: Extent3d {
                    width: l as u32,
                    height: l as u32,
                    depth_or_array_layers: l as u32,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: TextureFormat::R8Uint,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            bytemuck::cast_slice(&dataflat),
        );
        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some("Cells Texture View"),
            format: Some(TextureFormat::R8Uint),
            dimension: Some(wgpu::TextureViewDimension::D3),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                }],
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
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
        (compute_pipeline, bind_group)
    };

    let mut encoder =
        device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

    {
        let mut compute_pass =
            encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        compute_pass.dispatch_workgroups(l as u32, l as u32, l as u32);
    }
    queue.submit(Some(encoder.finish()));

    device.poll(wgpu::Maintain::Wait);
}
