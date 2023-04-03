mod camera;
mod cool_rules;
mod environment;
mod game_of_life;
mod instance;
mod model;
mod rule;
mod rule_parse;
mod texture;

use std::{fs, time::Instant};

use camera::Camera;
use environment::Environment;
use game_of_life::GameOfLife;
use model::{Model, Vertex};
use pollster::FutureExt;
use rule::Rule;
use wgpu::{
    include_wgsl, BlendState, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, DepthBiasState, DepthStencilState, Device,
    FragmentState, MultisampleState, Operations, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, StencilState,
    SurfaceConfiguration, TextureViewDescriptor, VertexState,
};
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Read the rule from a file
    #[arg(short, long)]
    file: Option<String>,

    /// Pass in the rule directly
    rule: Option<String>,
}

pub fn run() {
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

struct State {
    env: environment::Environment,
    camera: Camera,
    model: Model,
    instances: instance::InstancesVec,
    depth_texture: texture::Texture,
    render_pipeline: RenderPipeline,
    gol: GameOfLife,
    paused: bool,
}
impl State {
    fn new(window: Window) -> Self {
        //* GOL

        let args = Args::parse();

        let rule = {
            let mut rule_string = if let Some(r) = args.rule {
                r
            } else if let Some(f) = args.file {
                fs::read_to_string(f).unwrap()
            } else {
                cool_rules::as_str::PERIODIC_FUNKY.to_string()
            };

            rule_string.retain(|c| !c.is_whitespace());

            rule_string.parse::<Rule>().unwrap()
        };
        // let rule = cool_rules::as_rule::GLIDER_HEAVEN;
        // let rule = cool_rules::as_str::SHELLS.parse::<Rule>().urnwrap();
        // let rule = cool_rules::as_str::PERIODIC_FUNKY.parse::<Rule>().unwrap();

        let gol = GameOfLife {
            cells: GameOfLife::new_random_preset(rule.max_state),
            rule,
        };
        //* ENVIRONMENT
        let env = Environment::new(window).block_on();

        //* CAMERA
        let (camera, camera_bind_group_layout) =
            Camera::create_camera(&env.device, &env.config);

        //* MODEL
        let model = Model::new(&env.device, model::CUBE, model::CUBE_INDICES);
        let instances = instance::InstancesVec::from((&gol, &env.device));

        //* RENDERING
        let depth_texture =
            texture::Texture::create_depth_texture(&env.device, &env.config);
        let shader = env
            .device
            .create_shader_module(include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            env.device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let render_pipeline = Self::generate_render_pipeline(
            &env.device,
            &env.config,
            render_pipeline_layout,
            shader,
        );

        Self {
            env,
            camera,
            model,
            instances,
            depth_texture,
            render_pipeline,
            gol,
            paused: true,
        }
    }

    fn generate_render_pipeline(
        device: &Device,
        config: &SurfaceConfiguration,
        layout: PipelineLayout,
        shader: ShaderModule,
    ) -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                    instance::RawInstance::desc(),
                    // Vertex::desc(),
                ],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.env.size = new_size;
            self.env.config.width = new_size.width;
            self.env.config.height = new_size.height;
            self.env
                .surface
                .configure(&self.env.device, &self.env.config);
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.env.device,
                &self.env.config,
            );
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::Space)
                    && input.state == ElementState::Pressed =>
            {
                self.gol.update();
                self.instances = (&self.gol, &self.env.device).into();
                return true;
            }
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::Tab)
                    && input.state == ElementState::Released =>
            {
                self.paused = !self.paused;
                return true;
            }
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::R)
                    && input.state == ElementState::Released =>
            {
                self.gol.cells =
                    GameOfLife::new_random_preset(self.gol.rule.max_state);
                self.instances = (&self.gol, &self.env.device).into();
                return true;
            }
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::Q)
                    && input.state == ElementState::Released =>
            {
                self.gol.rule = Rule::new_random();
                self.gol.cells =
                    GameOfLife::new_random_preset(self.gol.rule.max_state);
                self.instances = (&self.gol, &self.env.device).into();
                println!("{}", self.gol.rule);
                return true;
            }
            _ => (),
        }
        self.camera.controller.process_events(event)
    }

    fn update(&mut self, delta: f32) {
        if !self.paused {
            self.gol.update();
            self.instances = (&self.gol, &self.env.device).into();
        }
        self.camera.update(delta);
        self.env.queue.write_buffer(
            &self.camera.buffer,
            0,
            bytemuck::cast_slice(&[self.camera.uniform.view_proj]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.env.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder =
            self.env
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_texture.view,
                            depth_ops: Some(Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        },
                    ),
                });
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass
                .set_vertex_buffer(0, self.model.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instances.buffer.slice(..));
            // render_pass.set_vertex_buffer(
            //     2,
            //     self.model.vertex_buffer.slice(..),
            // );
            render_pass.set_index_buffer(
                self.model.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(
                0..self.model.num_indices,
                0,
                0..self.instances.data.len() as _,
            );
        }
        self.env.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
