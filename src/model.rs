use wgpu::{util::DeviceExt, Buffer, BufferUsages, Device, VertexBufferLayout};

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

pub struct Model {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
}

impl Model {
    pub fn new(device: &Device) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: BufferUsages::VERTEX,
            },
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: BufferUsages::INDEX,
            },
        );

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len() as u32,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}
impl Vertex {
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
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
