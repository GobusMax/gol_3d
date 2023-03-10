use wgpu::{util::DeviceExt, Buffer, BufferUsages, Device, VertexBufferLayout};

pub const _PENTAGON: &[Vertex] = &[
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
pub const _PENTAGON_INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

pub const CUBE: &[Vertex] = &[
    //0
    Vertex {
        position: [0., 0., 0.],
        tex_coords: [0., 0.],
    },
    //1
    Vertex {
        position: [0., 0., 1.],
        tex_coords: [0., 1.],
    },
    //2
    Vertex {
        position: [0., 1., 0.],
        tex_coords: [1., 0.],
    },
    //3
    Vertex {
        position: [0., 1., 1.],
        tex_coords: [1., 1.],
    },
    //4
    Vertex {
        position: [1., 0., 0.],
        tex_coords: [0., 1.],
    },
    //5
    Vertex {
        position: [1., 0., 1.],
        tex_coords: [0., 0.],
    },
    //6
    Vertex {
        position: [1., 1., 0.],
        tex_coords: [1., 1.],
    },
    //7
    Vertex {
        position: [1., 1., 1.],
        tex_coords: [1., 0.],
    },
];
#[rustfmt::skip]
pub const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 
    3, 2, 1, 
    
    6, 5, 4, 
    5, 6, 7,
    
    5, 3, 1,
    3, 5, 7,
    
    0, 2, 4,
    6, 4, 2,
    
    4, 1, 0,
    1, 4, 5,
    
    2, 3, 6,
    7, 6, 3,
    
];

pub struct Model {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
}

impl Model {
    pub fn new(device: &Device, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            },
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::INDEX,
            },
        );

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
