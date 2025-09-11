use std::f64::consts::PI;
use nalgebra::{Isometry3, Matrix4, MatrixView4, Perspective3, Rotation3, SimdComplexField, Translation3, Unit, UnitQuaternion, Vector, Vector3};
use crate::toolbox::input::Input;
use glfw::{Key, MouseButton};

const CAMERA_SPEED: f64 = 0.1;
const CAMERA_ROTATION_SPEED: f64 = 0.001;

pub struct Camera {
    pub position: Vector3<f64>,
    pub quat: UnitQuaternion<f64>
}

impl Camera {
    pub fn new(position: Vector3<f64>) -> Camera {
        Camera {
            position,
            quat: UnitQuaternion::<f64>::from_axis_angle(&Vector3::y_axis(), 0.),
        }
    }

    pub fn update(&mut self, input: &Input) {
        self.update_angles(input);
        self.position += self.get_dp(input) * CAMERA_SPEED;
    }

    #[inline]
    fn update_angles(&mut self, input: &Input) {
        if input.is_mouse_button_pressed(MouseButton::Left) {
            let vec_dir = Vector3::new(input.d_mouse_pos.1, input.d_mouse_pos.0, 0.0);
            let norm = vec_dir.norm();
            let d_quat = if norm != 0.0 {;
                UnitQuaternion::from_axis_angle(&Unit::new_normalize(vec_dir), norm * CAMERA_ROTATION_SPEED)
            } else {
                UnitQuaternion::identity()
            };
            self.quat = self.quat * d_quat;
        }

    }

    #[inline]
    fn get_dp(&self, input: &Input) -> Vector3<f64> { // Use quaternion to remember previous rotation
        let front = self.quat.transform_vector(&Vector3::z_axis()).normalize();
        let up = self.quat.transform_vector(&Vector3::y_axis()).normalize();
        let right = self.quat.transform_vector(&Vector3::x_axis()).normalize();

        if input.is_key_pressed(Key::W) || input.is_key_pressed(Key::S) {
            let incr = if input.is_key_pressed(Key::W) { -1.0 } else { 1.0 };
            let dp = front * incr;
            dp
        } else if input.is_key_pressed(Key::A) || input.is_key_pressed(Key::D) {
            let incr = if input.is_key_pressed(Key::A) { -1.0 } else { 1.0 };
            let dp = right * incr;
            dp
        }else if input.is_key_pressed(Key::LeftShift) || input.is_key_pressed(Key::Space) {
            let incr = if input.is_key_pressed(Key::LeftShift) { -1.0 } else { 1.0 };
            let dp = up * incr;
            dp
        }else {
            Vector3::zeros()
        }
    }

    pub fn get_view_matrix(&self) -> Matrix4<f64>{
        let translation = Translation3::from(-self.position);
        let result = self.quat.inverse().to_homogeneous() * translation.to_homogeneous();
        result
    }
}