pub(crate) mod args;
pub(crate) mod camera;
mod compute_env;
pub(crate) mod cool_rules;
pub(crate) mod environment;
pub(crate) mod game_of_life;
pub(crate) mod instance;
pub(crate) mod model;
pub(crate) mod rule;
pub(crate) mod rule_parse;
pub(crate) mod texture;

use std::fs;

use camera::Camera;
use clap::Parser;
use compute_env::ComputeEnv;
use environment::Environment;
use game_of_life::{GameOfLife, SIZE};

use model::{Model, Vertex};

use pollster::FutureExt;
use rule::Rule;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BlendState, Buffer, BufferAddress,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, ComputePassDescriptor, DepthBiasState,
    DepthStencilState, Device, FragmentState, MultisampleState, Operations,
    PipelineLayout, PipelineLayoutDescriptor, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    StencilState, SurfaceConfiguration, TextureViewDescriptor, VertexState,
};
use winit::{
    event::{ElementState, VirtualKeyCode, WindowEvent},
    window::Window,
};

const WORKGROUP_SIZE: u32 = 4;

pub struct Init {
    pub size: usize,
    pub density: f64,
}

pub struct State {
    pub env: environment::Environment,
    pub camera: Camera,
    pub gol: GameOfLife,
    model: Model,
    instances: instance::InstancesVec,
    depth_texture: texture::Texture,
    render_pipeline: RenderPipeline,
    paused: bool,
    cursor_grab: bool,
    compute_env: ComputeEnv,
}
impl State {
    pub fn new(window: Window) -> Self {
        //* GOL

        let args = args::Args::parse();

        let (rule, mut init) = {
            let mut rule_string = if let Some(r) = args.rule {
                r
            } else if let Some(f) = args.file {
                fs::read_to_string(f).unwrap()
            } else {
                cool_rules::as_str::PERIODIC_FUNKY.to_string()
            };

            rule_string.retain(|c| !c.is_whitespace());

            rule_parse::rule_and_init(&rule_string).unwrap().1
        };

        if let Some(s) = args.init_size {
            init.size = s;
        }
        if let Some(d) = args.init_density {
            init.density = d;
        }

        let gol = GameOfLife {
            cells: GameOfLife::cells_random_init(rule.max_state, &init),
            rule,
            init,
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
        let draw_shader =
            env.device.create_shader_module(include_wgsl!("draw.wgsl"));
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
            &render_pipeline_layout,
            &draw_shader,
        );

        let compute_env = ComputeEnv::new(&gol, &env.device, &instances);

        Self {
            env,
            camera,
            model,
            instances,
            depth_texture,
            render_pipeline,
            gol,
            paused: true,
            cursor_grab: false,
            compute_env,
        }
    }
    fn generate_render_pipeline(
        device: &Device,
        config: &SurfaceConfiguration,
        layout: &PipelineLayout,
        shader: &ShaderModule,
    ) -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(layout),
            vertex: VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), instance::RawInstance::desc()],
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
                module: shader,
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

    fn update_cells_buffers(&mut self) {
        self.instances =
            instance::InstancesVec::from((&self.gol, &self.env.device));
        self.compute_env.num_instances = self.instances.data.len() as u32;
        (
            self.compute_env.bind_groups,
            self.compute_env.atomic_counter_buffer,
        ) = State::generate_cells_buffers_bind_group(self);
    }
    fn generate_cells_buffers_bind_group(&self) -> ([BindGroup; 2], Buffer) {
        let cells_vec: Vec<u32> = self
            .gol
            .cells
            .clone()
            .into_raw_vec()
            .iter()
            .map(|x| *x as u32)
            .collect();

        let mut buffers = Vec::with_capacity(2);
        for _i in 0..=1 {
            buffers.push(self.env.device.create_buffer_init(
                &BufferInitDescriptor {
                    label: Some("Cells Buffer {i}"),
                    contents: bytemuck::cast_slice(&cells_vec),
                    usage: BufferUsages::STORAGE,
                },
            ));
        }
        let atomic_counter_buffer =
            self.env.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(
                    &(self.instances.data.len() as u32),
                ),
                usage: BufferUsages::STORAGE
                    | BufferUsages::COPY_SRC
                    | BufferUsages::COPY_DST,
            });
        let bind_groups = (0..=1)
            .map(|i| {
                self.env
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Compute Bind Group {i}"),
                        layout: &self.compute_env.bind_groups_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(
                                    self.gol
                                        .rule
                                        .as_buffer(&self.env.device)
                                        .as_entire_buffer_binding(),
                                ),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Buffer(
                                    buffers[i].as_entire_buffer_binding(),
                                ),
                            },
                            BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Buffer(
                                    buffers[(i + 1) % 2]
                                        .as_entire_buffer_binding(),
                                ),
                            },
                            BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::Buffer(
                                    self.instances
                                        .buffer
                                        .as_entire_buffer_binding(),
                                ),
                            },
                            BindGroupEntry {
                                binding: 4,
                                resource: wgpu::BindingResource::Buffer(
                                    atomic_counter_buffer
                                        .as_entire_buffer_binding(),
                                ),
                            },
                        ],
                    })
            })
            .collect::<Vec<BindGroup>>()
            .try_into()
            .unwrap();
        (bind_groups, atomic_counter_buffer)
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

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::Return)
                    && input.state == ElementState::Released
                    && self.paused =>
            {
                self.update_game_call();
                return true;
            }
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::Space)
                    && input.state == ElementState::Released =>
            {
                self.paused = !self.paused;
                return true;
            }
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::R)
                    && input.state == ElementState::Released =>
            {
                self.gol.cells = GameOfLife::cells_random_init(
                    self.gol.rule.max_state,
                    &self.gol.init,
                );
                self.update_cells_buffers();
                return true;
            }
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::B)
                    && input.state == ElementState::Released =>
            {
                self.gol.rule = "1-8,11-12,17-31/12/2/M/100/1".parse().unwrap();
                self.gol.cells = GameOfLife::gol_2d_board(
                    SIZE,
                    SIZE,
                    1.,
                    self.gol.rule.max_state,
                );
                self.update_cells_buffers();
                return true;
            }
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::Q)
                    && input.state == ElementState::Released =>
            {
                self.gol.rule = Rule::new_random();
                self.gol.cells = GameOfLife::cells_random_init(
                    self.gol.rule.max_state,
                    &self.gol.init,
                );
                self.update_cells_buffers();
                println!("{}", self.gol.rule);
                self.env
                    .window
                    .set_title(&format!("Rule: {}", self.gol.rule));
                return true;
            }
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::Slash)
                    && input.state == ElementState::Released =>
            {
                if self.cursor_grab {
                    self.env
                        .window
                        .set_cursor_grab(winit::window::CursorGrabMode::None)
                        .unwrap();
                    self.env.window.set_cursor_visible(true);
                } else {
                    self.env
                        .window
                        .set_cursor_grab(
                            winit::window::CursorGrabMode::Confined,
                        )
                        .unwrap();
                    self.env.window.set_cursor_visible(false);
                }
                self.cursor_grab = !self.cursor_grab;
                return true;
            }
            _ => (),
        }
        self.camera.controller.process_events(event)
    }

    pub fn update(&mut self, delta: f32) {
        if self.cursor_grab {
            self.camera.update(delta);
            self.env.queue.write_buffer(
                &self.camera.buffer,
                0,
                bytemuck::cast_slice(&[self.camera.uniform.view_proj]),
            );
        }
        if self.paused {
            self.render_call();
        } else {
            self.update_game_call();
            self.render_call();
        }
    }

    fn update_game_call(&mut self) {
        let mut encoder =
            self.env
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Encoder"),
                });
        encoder.clear_buffer(
            &self.compute_env.atomic_counter_buffer,
            0,
            std::num::NonZeroU64::new(std::mem::size_of::<u32>() as u64),
        );
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("Compute Pass"),
                });
            compute_pass.set_pipeline(&self.compute_env.compute_pipeline);
            compute_pass.set_bind_group(
                0,
                &self.compute_env.bind_groups[self.compute_env.step_toggle],
                &[],
            );
            compute_pass.dispatch_workgroups(
                SIZE as u32 / WORKGROUP_SIZE,
                SIZE as u32 / WORKGROUP_SIZE,
                SIZE as u32 / WORKGROUP_SIZE,
            );
        }
        let staging_buffer = self.env.device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<u32>() as BufferAddress,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(
            &self.compute_env.atomic_counter_buffer,
            0,
            &staging_buffer,
            0,
            std::mem::size_of::<u32>() as u64,
        );
        self.env.queue.submit(Some(encoder.finish()));
        let slice = staging_buffer.slice(..);
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.env.device.poll(wgpu::Maintain::Wait);
        rx.receive().block_on().unwrap().unwrap();
        let data = slice.get_mapped_range();
        let res: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
        self.compute_env.num_instances = res[0];
        self.compute_env.step_toggle = (self.compute_env.step_toggle + 1) % 2;
    }

    fn render_call(&mut self) {
        let output = self.env.surface.get_current_texture().unwrap();
        let mut encoder =
            self.env
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Encoder"),
                });
        // self.test().block_on();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
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

            render_pass.set_index_buffer(
                self.model.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(
                0..self.model.num_indices,
                0,
                0..self.compute_env.num_instances,
            );
        }

        self.env.queue.submit(Some(encoder.finish()));
        output.present();
    }
}
