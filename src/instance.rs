use cgmath::{vec3, Vector3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use crate::game_of_life::{GameOfLife, SIZE};

pub struct InstancesVec {
    pub data: Vec<Instance>,
    pub raw: Vec<RawInstance>,
    pub buffer: Buffer,
}

impl From<(&GameOfLife, &Device)> for InstancesVec {
    fn from((gol, device): (&GameOfLife, &wgpu::Device)) -> Self {
        let instances: Vec<Instance> = gol
            .cells
            .indexed_iter()
            .filter_map(|(i, c)| {
                if *c != 0 {
                    Some(Instance {
                        position: vec3(i.0 as _, i.1 as _, i.2 as _),
                        state: *c as _,
                    })
                } else {
                    None
                }
            })
            .collect();
        let mut raw =
            instances.iter().map(RawInstance::new).collect::<Vec<_>>();
        raw.resize(
            SIZE * SIZE * SIZE,
            RawInstance {
                pos: [0.; 3],
                state: 0,
            },
        );

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&raw),
            usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
        });
        Self {
            data: instances,
            raw,
            buffer,
        }
    }
}

pub struct Instance {
    pub position: Vector3<f32>,
    pub state: u32,
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawInstance {
    pos: [f32; 3],
    state: u32,
}
impl RawInstance {
    pub fn new(instance: &Instance) -> Self {
        Self {
            pos: instance.position.into(),
            state: instance.state,
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<RawInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}
