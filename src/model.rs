use wgpu::{util::DeviceExt, Buffer, BufferUsages, Device, VertexBufferLayout};

const SQRT_3: f32 = 1.7320508;
pub const CUBE: &[Vertex] = &[
    //0
    Vertex {
        position: [0., 0., 0.],
        tex_coords: [0., 0.],
        normal: [-SQRT_3, -SQRT_3, -SQRT_3],
    },
    //1
    Vertex {
        position: [0., 0., 1.],
        tex_coords: [0., 1.],
        normal: [-SQRT_3, -SQRT_3, SQRT_3],
    },
    //2
    Vertex {
        position: [0., 1., 0.],
        tex_coords: [1., 0.],
        normal: [-SQRT_3, SQRT_3, -SQRT_3],
    },
    //3
    Vertex {
        position: [0., 1., 1.],
        tex_coords: [1., 1.],
        normal: [-SQRT_3, SQRT_3, SQRT_3],
    },
    //4
    Vertex {
        position: [1., 0., 0.],
        tex_coords: [0., 1.],
        normal: [SQRT_3, -SQRT_3, -SQRT_3],
    },
    //5
    Vertex {
        position: [1., 0., 1.],
        tex_coords: [0., 0.],
        normal: [SQRT_3, -SQRT_3, SQRT_3],
    },
    //6
    Vertex {
        position: [1., 1., 0.],
        tex_coords: [1., 1.],
        normal: [SQRT_3, SQRT_3, -SQRT_3],
    },
    //7
    Vertex {
        position: [1., 1., 1.],
        tex_coords: [1., 0.],
        normal: [SQRT_3, SQRT_3, SQRT_3],
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
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
