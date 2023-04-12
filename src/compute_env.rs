use ndarray::Array3;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferUsages, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, ShaderModule,
    ShaderStages,
};

use crate::{game_of_life::GameOfLife, instance::InstancesVec, rule::Rule};

pub struct ComputeEnv {
    pub(crate) bind_groups_layout: BindGroupLayout,
    pub(crate) bind_groups: [BindGroup; 2],
    pub(crate) compute_pipeline: ComputePipeline,
    pub(crate) step_toggle: usize,
    pub(crate) atomic_counter_buffer: Buffer,
    pub(crate) num_instances: u32,
    pub(crate) _compute_shader: ShaderModule,
}

impl ComputeEnv {
    pub fn new(
        gol: &GameOfLife,
        device: &Device,
        instances: &InstancesVec,
    ) -> Self {
        let bind_groups_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Cell Buffer Bind Group Layout"),
                entries: &[
                    //RULE
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
                    //CELLS IN
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
                    //CELLS IN
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
                    //INSTANCES
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
                    //ATOMIC_COUNTER
                    BindGroupLayoutEntry {
                        binding: 4,
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
        let (bind_groups, atomic_counter_buffer) =
            Self::generate_cells_buffers_bind_group(
                &gol.cells,
                device,
                &bind_groups_layout,
                instances,
                &gol.rule,
            );
        let compute_pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&bind_groups_layout],
                push_constant_ranges: &[],
            });
        let compute_shader =
            device.create_shader_module(include_wgsl!("compute.wgsl"));
        let compute_pipeline =
            device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: Some(&compute_pipeline_layout),
                module: &compute_shader,
                entry_point: "cs_main",
            });
        Self {
            bind_groups_layout,
            bind_groups,
            compute_pipeline,
            step_toggle: 0,
            atomic_counter_buffer,
            num_instances: instances.data.len() as u32,
            _compute_shader: compute_shader,
        }
    }
    fn generate_cells_buffers_bind_group(
        cells: &Array3<u8>,
        device: &Device,
        compute_bind_groups_layout: &BindGroupLayout,
        instances: &InstancesVec,
        rule: &Rule,
    ) -> ([BindGroup; 2], Buffer) {
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
        let atomic_counter_buffer =
            device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&(instances.data.len() as u32)),
                usage: BufferUsages::STORAGE
                    | BufferUsages::COPY_SRC
                    | BufferUsages::COPY_DST,
            });
        let bind_groups = (0..=1)
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
                                instances.buffer.as_entire_buffer_binding(),
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
}
