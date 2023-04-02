use cgmath::{prelude::*, vec3, vec4, Matrix4, Quaternion, Vector3, Vector4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use crate::game_of_life::GameOfLife;

pub struct InstancesVec {
    pub data: Vec<Instance>,
    pub raw: Vec<RawInstance>,
    pub buffer: Buffer,
}

impl
    From<(
        &GameOfLife,
        &Device,
    )> for InstancesVec
{
    fn from(
        (gol, device): (
            &GameOfLife,
            &wgpu::Device,
        ),
    ) -> Self {
        let instances: Vec<Instance> = gol
            .cells
            .indexed_iter()
            .filter_map(
                |(i, c)| {
                    if *c == 0 {
                        None
                    } else {
                        Some(
                            Instance {
                                position: vec3(
                                    i.0 as _, i.1 as _, i.2 as _,
                                ) * 0.5,
                                rotation: Quaternion::zero(),
                                color: vec4(
                                    *c as f32 / gol.rule.max_state as f32,
                                    0.,
                                    0.,
                                    1.,
                                ),
                            },
                        )
                    }
                },
            )
            .collect();
        let raw = instances.iter().map(RawInstance::new).collect::<Vec<_>>();
        let buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&raw),
                usage: BufferUsages::VERTEX,
            },
        );
        Self {
            data: instances,
            raw,
            buffer,
        }
    }
}

pub struct Instance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub color: Vector4<f32>,
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawInstance {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}
impl RawInstance {
    pub fn new(instance: &Instance) -> Self {
        Self {
            model: (Matrix4::from_translation(instance.position)
                * Matrix4::from_scale(0.5)
                * Matrix4::from(instance.rotation))
            .into(),
            color: instance.color.into(),
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
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
