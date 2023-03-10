mod camera;
mod environment;
mod instance;
mod model;
mod texture;

use camera::Camera;
use environment::Environment;
use model::{Model, Vertex};
use wgpu::{
    include_wgsl, BindGroup, BlendState, ColorTargetState, ColorWrites, CommandEncoderDescriptor,
    DepthBiasState, DepthStencilState, FragmentState, MultisampleState, Operations,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, StencilState, TextureViewDescriptor, VertexState,
};
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(
            winit::dpi::PhysicalSize::new(
                1600, 900,
            ),
        )
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(window).await;
    state
        .env
        .window
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();
    state.env.window.set_cursor_visible(false);
    event_loop.run(
        move |event, _, control_flow| match event {
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
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.env.window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.env.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!(
                        "{:?}",
                        e
                    ),
                }
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
        },
    );
}

struct State {
    env: environment::Environment,
    texture_bind_group: BindGroup,
    _texture: texture::Texture,
    camera: Camera,
    model: Model,
    instances: instance::InstancesVec,
    depth_texture: texture::Texture,
    render_pipeline: RenderPipeline,
}
impl State {
    async fn new(window: Window) -> Self {
        //* ENVIRONMENT
        let env = Environment::new(window).await;

        //* TEXTURE
        let diffuse_bytes = include_bytes!("happy-tree.png");
        let (texture, texture_bind_group_layout, texture_bind_group) =
            texture::Texture::from_bytes(
                &env.device,
                &env.queue,
                diffuse_bytes,
                "Texture",
            )
            .unwrap();

        //* CAMERA
        let (camera, camera_bind_group_layout) = Camera::create_camera(
            &env.device,
            &env.config,
        );

        //* MODEL
        let model = Model::new(&env.device);
        let instances = instance::InstancesVec::new(&env.device);

        //* RENDERING
        let depth_texture = texture::Texture::create_depth_texture(
            &env.device,
            &env.config,
            Some("Depth Texture"),
        );
        let shader = env
            .device
            .create_shader_module(include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = env.device.create_pipeline_layout(
            &PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            },
        );
        let render_pipeline = env.device.create_render_pipeline(
            &RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), instance::RawInstance::desc()],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(
                    DepthStencilState {
                        format: texture::Texture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: StencilState::default(),
                        bias: DepthBiasState::default(),
                    },
                ),
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(
                    FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(
                            ColorTargetState {
                                format: env.config.format,
                                blend: Some(BlendState::ALPHA_BLENDING),
                                write_mask: ColorWrites::ALL,
                            },
                        )],
                    },
                ),
                multiview: None,
            },
        );

        Self {
            env,
            texture_bind_group,
            _texture: texture,
            camera,
            model,
            instances,
            depth_texture,
            render_pipeline,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.env.size = new_size;
            self.env.config.width = new_size.width;
            self.env.config.height = new_size.height;
            self.env.surface.configure(
                &self.env.device,
                &self.env.config,
            );
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.env.device,
                &self.env.config,
                Some("Depth Texture"),
            )
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera.controller.process_events(event)
    }

    fn update(&mut self) {
        self.camera.update();
        self.env.queue.write_buffer(
            &self.camera.buffer,
            0,
            bytemuck::cast_slice(&[self.camera.uniform.view_proj]),
        )
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.env.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self.env.device.create_command_encoder(
            &CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            },
        );
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: 0.1,
                                        g: 0.2,
                                        b: 0.3,
                                        a: 1.0,
                                    },
                                ),
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_texture.view,
                            depth_ops: Some(
                                Operations {
                                    load: wgpu::LoadOp::Clear(1.0),
                                    store: true,
                                },
                            ),
                            stencil_ops: None,
                        },
                    ),
                },
            );
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(
                0,
                &self.texture_bind_group,
                &[],
            );
            render_pass.set_bind_group(
                1,
                &self.camera.bind_group,
                &[],
            );
            render_pass.set_vertex_buffer(
                0,
                self.model.vertex_buffer.slice(..),
            );
            render_pass.set_vertex_buffer(
                1,
                self.instances.buffer.slice(..),
            );
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
