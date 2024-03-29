use cgmath::{
    perspective, prelude::*, Deg, Matrix3, Matrix4, Point3, Rad, Vector3,
};
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutEntry, Buffer, BufferUsages, Device, ShaderStages,
    SurfaceConfiguration,
};
use winit::event::{
    DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, WindowEvent,
};

use crate::game_of_life::SIZE;

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5,
    1.0,
);

pub struct Camera {
    pub entity: CameraEntity,
    pub uniform: CameraUniform,
    pub controller: CameraController,
    pub bind_group: BindGroup,
    pub buffer: Buffer,
}

impl Camera {
    pub fn create_camera(
        device: &Device,
        config: &SurfaceConfiguration,
    ) -> (Self, BindGroupLayout) {
        let entity = CameraEntity {
            pos: (SIZE as f32, SIZE as f32, SIZE as f32).into(),
            dir: Vector3::from((-1., -1., -1.)).normalize(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
        };
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&entity);
        let buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&uniform.view_proj),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Groups"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        let controller = CameraController::new(SIZE as f32 * 1., 0.001);

        (
            Self {
                entity,
                uniform,
                controller,
                bind_group,
                buffer,
            },
            bind_group_layout,
        )
    }

    pub fn update(&mut self, delta: f32) {
        self.controller
            .update_camera_entity(&mut self.entity, delta);
        self.uniform.update_view_proj(&self.entity);
    }
}
pub struct CameraEntity {
    pub pos: Point3<f32>,
    pub dir: Vector3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl CameraEntity {
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_to_rh(self.pos, self.dir, self.up);
        let proj =
            perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
    pub fn update_view_proj(&mut self, camera_entity: &CameraEntity) {
        self.view_proj = camera_entity.build_view_projection_matrix().into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}
pub struct CameraController {
    speed: f32,
    sens: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    delta: (f32, f32),
}

impl CameraController {
    pub fn new(speed: f32, sens: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            delta: (0., 0.),
            sens,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
    pub fn process_mouse(&mut self, event: &DeviceEvent) -> bool {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.delta = (delta.0 as f32, delta.1 as f32);
            true
        } else {
            false
        }
    }
    pub fn update_camera_entity(
        &mut self,
        camera_entity: &mut CameraEntity,
        delta: f32,
    ) {
        camera_entity.dir = camera_entity.dir.normalize();
        let yaw = Matrix3::from_angle_y(Rad(-self.delta.0) * self.sens);
        camera_entity.dir = yaw * camera_entity.dir;

        let pitch = Matrix3::from_axis_angle(
            camera_entity.dir.cross(camera_entity.up).normalize(),
            -Rad(self.delta.1) * self.sens,
        );
        self.delta = (0., 0.);
        let new_dir = pitch * camera_entity.dir;
        if camera_entity
            .dir
            .cross(camera_entity.up)
            .dot(new_dir.cross(camera_entity.up))
            >= 0.
        {
            camera_entity.dir = new_dir;
        } else {
            camera_entity.dir.y = camera_entity.dir.y.signum();
        }
        camera_entity.dir = camera_entity.dir.normalize();
        let forward = camera_entity.dir * self.speed * delta;
        if self.is_forward_pressed {
            camera_entity.pos += forward;
        }
        if self.is_backward_pressed {
            camera_entity.pos -= forward;
        }

        let right = camera_entity.dir.cross(camera_entity.up).normalize()
            * self.speed
            * delta;

        if self.is_right_pressed {
            camera_entity.pos += right;
        }
        if self.is_left_pressed {
            camera_entity.pos -= right;
        }
    }
}
