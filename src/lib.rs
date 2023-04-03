pub mod camera;
pub mod cool_rules;
pub mod environment;
pub mod game_of_life;
pub mod instance;
pub mod model;
pub mod rule;
pub mod rule_parse;
pub mod texture;
pub mod args;

use std::fs;

use camera::Camera;
use clap::Parser;
use environment::Environment;
use game_of_life::GameOfLife;
use model::{Model, Vertex};
use pollster::FutureExt;
use rule::Rule;
use wgpu::{
    include_wgsl, BlendState, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, DepthBiasState, DepthStencilState, Device,
    FragmentState, MultisampleState, Operations, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, StencilState,
    SurfaceConfiguration, TextureViewDescriptor, VertexState,
};
use winit::{
    event::{ElementState, VirtualKeyCode, WindowEvent},
    window::Window,
};

pub struct Init {
    pub size: usize,
    pub density: f64,
}

pub struct State {
    pub env: environment::Environment,
    pub camera: Camera,
    pub model: Model,
    pub instances: instance::InstancesVec,
    pub depth_texture: texture::Texture,
    pub render_pipeline: RenderPipeline,
    pub compute_pipeline: ComputePipeline,
    pub gol: GameOfLife,
    pub paused: bool,
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
            &shader,
        );

        let compute_pipeline =
            Self::generate_compute_pipeline(&env.device, &shader);
        println!("{:?}", env.device.limits());
        Self {
            env,
            camera,
            model,
            instances,
            depth_texture,
            render_pipeline,
            compute_pipeline,
            gol,
            paused: true,
        }
    }
    fn generate_compute_pipeline(
        device: &Device,
        shader: &ShaderModule,
    ) -> ComputePipeline {
        let compute_pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: shader,
            entry_point: "cs_main",
        })
    }
    fn generate_render_pipeline(
        device: &Device,
        config: &SurfaceConfiguration,
        layout: PipelineLayout,
        shader: &ShaderModule,
    ) -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
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
                    && input.state == ElementState::Pressed =>
            {
                self.gol.update();
                self.instances = (&self.gol, &self.env.device).into();
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
                self.instances = (&self.gol, &self.env.device).into();
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
                self.instances = (&self.gol, &self.env.device).into();
                println!("{}", self.gol.rule);
                return true;
            }
            _ => (),
        }
        self.camera.controller.process_events(event)
    }

    pub fn update(&mut self, delta: f32) {
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

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            let mut compute_pass =
                encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("Compute Pass"),
                });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.dispatch_workgroups(10, 10, 10);
        }
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
                0..self.instances.data.len() as _,
            );
        }
        self.env.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
