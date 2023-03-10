use cgmath::{prelude::*, vec3, Matrix4, Quaternion, Vector3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

pub const INSTANCES_PER_ROW: i32 = 10;
pub const INSTANCE_DISPLACEMENT: Vector3<f32> = vec3(
    INSTANCES_PER_ROW as f32 * 0.5,
    0.,
    INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct InstancesVec {
    pub data: Vec<Instance>,
    pub raw: Vec<RawInstance>,
    pub buffer: Buffer,
}

impl InstancesVec {
    pub fn new(device: &Device) -> Self {
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

        let raw_instances = instances.iter().map(RawInstance::new).collect::<Vec<_>>();
        let buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&raw_instances),
                usage: BufferUsages::VERTEX,
            },
        );
        Self {
            data: instances,
            raw: raw_instances,
            buffer,
        }
    }
}
pub struct Instance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawInstance {
    model: [[f32; 4]; 4],
}
impl RawInstance {
    pub fn new(instance: &Instance) -> Self {
        Self {
            model: (Matrix4::from_translation(instance.position)
                * Matrix4::from(instance.rotation))
            .into(),
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
            ],
        }
    }
}
