use cgmath::{perspective, prelude::*, Deg, Matrix3, Matrix4, Point3, Rad, Vector3};
use winit::event::*;

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub eye: Point3<f32>,
    pub dir: Vector3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_to_rh(
            self.eye, self.dir, self.up,
        );
        let proj = perspective(
            Deg(self.fovy),
            self.aspect,
            self.znear,
            self.zfar,
        );
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
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
pub struct CameraController {
    speed: f32,
    sens: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    delta: (
        f32,
        f32,
    ),
}

impl CameraController {
    pub fn new(speed: f32, sens: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            delta: (
                0., 0.,
            ),
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
            self.delta = (
                delta.0 as f32,
                delta.1 as f32,
            );
            true
        } else {
            false
        }
    }
    pub fn update_camera(&mut self, camera: &mut Camera) {
        camera.dir = camera.dir.normalize();
        let yaw = Matrix3::from_angle_y(Rad(-self.delta.0) * self.sens);
        camera.dir = yaw * camera.dir;

        let pitch = Matrix3::from_axis_angle(
            camera.dir.cross(camera.up).normalize(),
            -Rad(self.delta.1) * self.sens,
        );
        self.delta = (
            0., 0.,
        );
        let new_dir = pitch * camera.dir;
        if camera.dir.cross(camera.up).dot(new_dir.cross(camera.up)) >= 0. {
            camera.dir = new_dir;
        } else {
            camera.dir.y = camera.dir.y.signum();
        }
        camera.dir = camera.dir.normalize();
        if self.is_forward_pressed {
            camera.eye += camera.dir * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= camera.dir * self.speed;
        }

        let right = camera.dir.cross(camera.up).normalize();

        if self.is_right_pressed {
            camera.eye += right * self.speed;
        }
        if self.is_left_pressed {
            camera.eye += right * -self.speed;
        }
    }
}
