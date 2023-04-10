use wgpu::{util::DeviceExt, Buffer, BufferUsages, Device, VertexBufferLayout};

pub const CUBE: &[Vertex] = &[
    // *0
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [0., 0., -1.],
    },
    // *1
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [0., 0., -1.],
    },
    // *2
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [0., 0., -1.],
    },
    // *3
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [0., 0., -1.],
    },
    // *
    // *4
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [0., 0., 1.],
    },
    // *5
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [0., 0., 1.],
    },
    // *6
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [0., 0., 1.],
    },
    // *7
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [0., 0., 1.],
    },
    // *
    // *8
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [-1., 0., 0.],
    },
    // *9
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [-1., 0., 0.],
    },
    // *10
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [-1., 0., 0.],
    },
    // *11
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [-1., 0., 0.],
    },
    // *
    // *12
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [1., 0., 0.],
    },
    // *13
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [1., 0., 0.],
    },
    // *14
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [1., 0., 0.],
    },
    // *15
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [1., 0., 0.],
    },
    // *
    // *16
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [0., -1., 0.],
    },
    // *17
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [0., -1., 0.],
    },
    // *18
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [0., -1., 0.],
    },
    // *19
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [0., -1., 0.],
    },
    // *
    // *20
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [0., 1., 0.],
    },
    // *21
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [0., 1., 0.],
    },
    // *22
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [0., 1., 0.],
    },
    // *23
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [0., 1., 0.],
    },
];
#[rustfmt::skip]
pub const CUBE_INDICES: &[u16] = &[
    0, 2, 1,
    3, 1, 2,

    6, 4, 5,
    5, 7, 6,

    8, 10, 9,
    11, 9, 10,

    14, 12, 13,
    13, 15, 14,

    16, 18, 17,
    19, 17, 18,

    22, 20, 21,
    21, 23, 22,

];
pub struct Model {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
}

impl Model {
    pub fn new(device: &Device, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            });
        let index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::INDEX,
            });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
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
                    offset: std::mem::size_of::<[f32; 3]>()
                        as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
