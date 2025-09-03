
use nalgebra::{Isometry3, Matrix4, MatrixView4, Perspective3, Rotation3, SimdComplexField, Translation3, UnitQuaternion, Vector3};
use crate::toolbox::input::Input;
use glfw::{Key, MouseButton};

const CAMERA_SPEED: f64 = 10.0;
const CAMERA_ROTATION_SPEED: f64 = 0.01;

pub struct Camera {
    pub position: Vector3<f64>,
    pub yaw: f64,   // Angle in the local plane Ox, Oy (in radians)
    pub pitch: f64, // Angle in the local plane Ox, Oz (in radians)
    pub roll: f64,  // Angle in the local plane Oy, Oz (in radians)
    pub fov: f64,
}

impl Camera {
    pub fn new(position: Vector3<f64>, yaw: f64, pitch: f64, roll: f64, fov: f64) -> Camera {
        Camera {
            position,
            yaw,
            pitch,
            roll,
            fov,
        }
    }

    pub fn update(&mut self, input: &Input) {
        self.update_angles(input);
        self.position += self.get_dp(input) * CAMERA_SPEED;
    }

    #[inline]
    fn update_angles(&mut self, input: &Input) {
        if input.is_mouse_button_pressed(MouseButton::Left) {
            self.yaw += -(input.d_mouse_pos.0 as f64) * CAMERA_ROTATION_SPEED;
            self.pitch += -(input.d_mouse_pos.1 as f64) * CAMERA_ROTATION_SPEED
        }
    }

    #[inline]
    fn get_dp(&self, input: &Input) -> Vector3<f64> {
        let mut dp: Vector3<f64> = Vector3::new(0.0, 0.0, 0.0);
        if input.is_key_pressed(Key::W) || input.is_key_pressed(Key::S) {
            let incr = if input.is_key_pressed(Key::W) { 1.0 } else { -1.0 };
            dp += Vector3::new(incr, 0., 0.) * self.yaw.cos() * self.pitch.cos();
            dp += Vector3::new(0., incr, 0.) * self.yaw.sin() * self.pitch.cos();
            dp += Vector3::new(0., 0., incr) * self.pitch.sin();
        };
        if input.is_key_pressed(Key::A) || input.is_key_pressed(Key::D) {
            let incr = if input.is_key_pressed(Key::A) { 1.0 } else { -1.0 };
            dp += Vector3::new(incr, 0., 0.) * -self.yaw.sin();
            dp += Vector3::new(0., incr, 0.) * self.yaw.cos();
            dp += Vector3::new(0., 0., incr) * -self.yaw.sin() * self.pitch.sin();
        }
        if input.is_key_pressed(Key::LeftShift) || input.is_key_pressed(Key::Space) {
            let incr = if input.is_key_pressed(Key::LeftShift) { -1.0 } else { 1.0 };
            dp += Vector3::new(incr, 0., 0.) * self.yaw.cos() * -self.pitch.sin();
            dp += Vector3::new(0., incr, 0.) * self.yaw.sin() * -self.pitch.sin();
            dp += Vector3::new(0., 0., incr) * self.pitch.cos();
        }
        dp
    }

    pub fn get_view_matrix(&self) -> Matrix4<f64>{
        let rotation = Rotation3::from_euler_angles(self.roll, self.pitch, self.yaw);
        let translation = Translation3::from(self.position);
        Isometry3::from_parts(translation, <UnitQuaternion<f64>>::from(rotation)).to_homogeneous()
    }
}