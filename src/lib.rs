mod camera;
mod environment;
mod texture;

use camera::{CameraController, CameraEntity, CameraUniform};

use cgmath::{prelude::*, vec3, Matrix4, Quaternion, Vector3};
use environment::Environment;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayoutEntry, BlendState, Buffer, BufferUsages,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, DepthBiasState, DepthStencilState,
    FragmentState, MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StencilState,
    TextureViewDescriptor, VertexBufferLayout, VertexState,
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
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}
impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.0, 0.49240386],
        tex_coords: [0.4131759, 0.99240386],
    },
    Vertex {
        position: [-0.49513406, 0.0, 0.06958647],
        tex_coords: [0.0048659444, 0.56958647],
    },
    Vertex {
        position: [-0.21918549, 0.0, -0.44939706],
        tex_coords: [0.28081453, 0.05060294],
    },
    Vertex {
        position: [0.35966998, 0.0, -0.3473291],
        tex_coords: [0.85967, 0.1526709],
    },
    Vertex {
        position: [0.44147372, 0.0, 0.2347359],
        tex_coords: [0.9414737, 0.7347359],
    },
];

const INDICES: &[u16; 9] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
const INSTANCES_PER_ROW: i32 = 10;
const INSTANCE_DISPLACEMENT: Vector3<f32> = vec3(
    INSTANCES_PER_ROW as f32 * 0.5,
    0.,
    INSTANCES_PER_ROW as f32 * 0.5,
);
struct Instance {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
}
impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation)).into(),
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}
impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

struct State {
    env: environment::Environment,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    texture_bind_group: BindGroup,
    _texture: texture::Texture,
    camera: camera::Camera,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
    depth_texture: texture::Texture,
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
        let camera_entity = CameraEntity {
            eye: (
                0.0, 2.0, 0.0,
            )
                .into(),
            dir: (
                0.1, -1.0, 0.1,
            )
                .into(),
            up: cgmath::Vector3::unit_y(),
            aspect: env.config.width as f32 / env.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera_entity);
        let camera_buffer = env.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&camera_uniform.view_proj),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            },
        );
        let camera_bind_group_layout = env.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        );
        let camera_bind_group = env.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Groups"),
                layout: &camera_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            },
        );
        let camera_controller = CameraController::new(
            0.01, 0.001,
        );
        let camera = camera::Camera {
            entity: camera_entity,
            uniform: camera_uniform,
            controller: camera_controller,
            bind_group: camera_bind_group,
            buffer: camera_buffer,
        };
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
                    buffers: &[Vertex::desc(), InstanceRaw::desc()],
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

        //* MODEL
        let vertex_buffer = env.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: BufferUsages::VERTEX,
            },
        );
        let index_buffer = env.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: BufferUsages::INDEX,
            },
        );

        let instances = (0..INSTANCES_PER_ROW)
            .flat_map(
                |i| {
                    (0..INSTANCES_PER_ROW).map(
                        move |j| {
                            let position = vec3(
                                i as f32, 0., j as f32,
                            ) - INSTANCE_DISPLACEMENT;

                            let rotation = if position.is_zero() {
                                cgmath::Quaternion::from_axis_angle(
                                    cgmath::Vector3::unit_z(),
                                    cgmath::Deg(0.0),
                                )
                            } else {
                                cgmath::Quaternion::from_axis_angle(
                                    position.normalize(),
                                    cgmath::Deg(45.0),
                                )
                            };
                            Instance { position, rotation }
                        },
                    )
                },
            )
            .collect::<Vec<_>>();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = env.device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: BufferUsages::VERTEX,
            },
        );
        Self {
            env,
            render_pipeline,
            vertex_buffer,
            num_indices: INDICES.len() as u32,
            index_buffer,
            _texture: texture,
            camera,
            instances,
            instance_buffer,
            depth_texture,
            texture_bind_group,
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
                self.vertex_buffer.slice(..),
            );
            render_pass.set_vertex_buffer(
                1,
                self.instance_buffer.slice(..),
            );
            render_pass.set_index_buffer(
                self.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(
                0..self.num_indices,
                0,
                0..self.instances.len() as _,
            );
        }
        self.env.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
