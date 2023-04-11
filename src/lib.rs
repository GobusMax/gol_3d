pub(crate) mod args;
pub(crate) mod camera;
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
use environment::Environment;
use game_of_life::{GameOfLife, SIZE};
use model::{Model, Vertex};
use ndarray::Array3;
use pollster::FutureExt;
use rule::Rule;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BlendState, Buffer, BufferUsages, ColorTargetState,
    ColorWrites, CommandEncoder, CommandEncoderDescriptor,
    ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor,
    DepthBiasState, DepthStencilState, Device, FragmentState, MultisampleState,
    Operations, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderStages, StencilState, SurfaceConfiguration, SurfaceTexture,
    TextureViewDescriptor, VertexState,
};
use winit::{
    event::{ElementState, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
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
    compute_bind_groups_layout: BindGroupLayout,
    compute_bind_groups: [BindGroup; 2],
    compute_pipeline: ComputePipeline,
    step_toggle: usize,
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
        // let rule = cool_rules::as_rule::GLIDER_HEAVEN;
        // let rule = cool_rules::as_str::SHELLS.parse::<Rule>().urnwrap();
        // let rule = cool_rules::as_str::PERIODIC_FUNKY.parse::<Rule>().unwrap();

        // println!("{}/{}/{}", rule, init.size, init.density);

        let gol = GameOfLife {
            cells: GameOfLife::cells_random_init(rule.max_state, &init),
            rule: rule.clone(),
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
        let compute_shader = env
            .device
            .create_shader_module(include_wgsl!("compute.wgsl"));
        let (compute_pipeline, compute_bind_groups, compute_bind_groups_layout) =
            Self::init_compute(
                &gol.cells,
                &env.device,
                &compute_shader,
                &instances.buffer,
                &rule,
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
            cursor_grab: true,
            compute_bind_groups_layout,
            compute_bind_groups,
            compute_pipeline,
            step_toggle: 0,
        }
    }
    fn init_compute(
        cells: &Array3<u8>,
        device: &Device,
        shader: &ShaderModule,
        instance_buffer: &Buffer,
        rule: &Rule,
    ) -> (ComputePipeline, [BindGroup; 2], BindGroupLayout) {
        let bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Cell Buffer Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: true,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: false,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: false,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let bind_groups = Self::generate_cells_buffers_bind_group(
            cells,
            device,
            &bind_group_layout,
            instance_buffer,
            rule,
        );
        let compute_pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline =
            device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: Some(&compute_pipeline_layout),
                module: shader,
                entry_point: "cs_main",
            });
        (compute_pipeline, bind_groups, bind_group_layout)
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

        self.compute_bind_groups = State::generate_cells_buffers_bind_group(
            &self.gol.cells,
            &self.env.device,
            &self.compute_bind_groups_layout,
            &self.instances.buffer,
            &self.gol.rule,
        );
    }
    fn generate_cells_buffers_bind_group(
        cells: &Array3<u8>,
        device: &Device,
        compute_bind_groups_layout: &BindGroupLayout,
        instance_buffer: &Buffer,
        rule: &Rule,
    ) -> [BindGroup; 2] {
        let cells_vec: Vec<u32> = cells
            .clone()
            .into_raw_vec()
            .iter()
            .map(|x| *x as u32)
            .collect();

        let mut buffers = Vec::with_capacity(2);
        for _i in 0..=1 {
            buffers.push(device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Cells Buffer {i}"),
                contents: bytemuck::cast_slice(&cells_vec),
                usage: BufferUsages::STORAGE,
            }));
        }

        (0..=1)
            .map(|i| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Compute Bind Group {i}"),
                    layout: compute_bind_groups_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(
                                rule.as_buffer(device)
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
                                buffers[(i + 1) % 2].as_entire_buffer_binding(),
                            ),
                        },
                        BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Buffer(
                                instance_buffer.as_entire_buffer_binding(),
                            ),
                        },
                    ],
                })
            })
            .collect::<Vec<BindGroup>>()
            .try_into()
            .unwrap()
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
                self.update_game_only();
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

    pub fn update(&mut self, delta: f32, control_flow: &mut ControlFlow) {
        if self.cursor_grab {
            self.camera.update(delta);
            self.env.queue.write_buffer(
                &self.camera.buffer,
                0,
                bytemuck::cast_slice(&[self.camera.uniform.view_proj]),
            );
        }
        if self.paused {
            match self.render_only() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => self.resize(self.env.size),
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    *control_flow = ControlFlow::Exit;
                }
                Err(e) => eprintln!("{e:?}"),
            }
        } else {
            match self.update_game_and_render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => self.resize(self.env.size),
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    *control_flow = ControlFlow::Exit;
                }
                Err(e) => eprintln!("{e:?}"),
            }
        }
    }

    fn update_game_only(&mut self) {
        let mut encoder =
            self.env
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Encoder"),
                });
        self.update_game_call(&mut encoder);
        self.env.queue.submit(Some(encoder.finish()));
    }
    fn update_game_call(&mut self, encoder: &mut CommandEncoder) {
        let mut compute_pass =
            encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(
            0,
            &self.compute_bind_groups[self.step_toggle],
            &[],
        );
        //TODO
        compute_pass.dispatch_workgroups(
            SIZE as u32 / WORKGROUP_SIZE,
            SIZE as u32 / WORKGROUP_SIZE,
            SIZE as u32 / WORKGROUP_SIZE,
        );
        self.step_toggle = (self.step_toggle + 1) % 2;
    }
    fn update_game_and_render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.env.surface.get_current_texture()?;
        let mut encoder =
            self.env
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Encoder"),
                });

        self.update_game_call(&mut encoder);

        self.render_call(&mut encoder, &output);

        self.env.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
    fn render_only(&mut self) -> Result<(), wgpu::SurfaceError> {
        let mut encoder =
            self.env
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Encoder"),
                });
        let output = self.env.surface.get_current_texture()?;

        self.render_call(&mut encoder, &output);
        self.env.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
    fn render_call(
        &mut self,
        encoder: &mut CommandEncoder,
        output: &SurfaceTexture,
    ) {
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
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
                })],
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
        render_pass.set_vertex_buffer(0, self.model.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.buffer.slice(..));

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
}
