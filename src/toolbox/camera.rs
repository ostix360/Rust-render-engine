use crate::toolbox::input::Input;
use glfw::{Key, MouseButton};
use nalgebra::{Matrix4, Translation3, Unit, UnitQuaternion, Vector3, Vector4};
use std::f64::consts::PI;
use crate::toolbox::opengl::display_manager::DisplayManager;

const CAMERA_SPEED: f64 = 0.1;
const CAMERA_ROTATION_SPEED: f64 = 0.001;

pub struct Camera {
    pub position: Vector3<f64>,
    pub quat: UnitQuaternion<f64>,
    pub pitch: f64,
}

impl Camera {
    pub fn new(position: Vector3<f64>) -> Camera {
        Camera {
            position,
            quat: UnitQuaternion::<f64>::from_axis_angle(&Vector3::x_axis(), -PI/2.),
            pitch: 0.,
        }
    }

    pub fn update(&mut self, input: &Input) {
        self.update_angles(input);
        self.position += self.get_dp(input) * CAMERA_SPEED;
    }

    pub fn mouse_pos_to_world_pos(&self, dp: &DisplayManager, projection: Matrix4<f64>) -> (Vector3<f64>, Vector3<f64>) {
        let input = dp.get_input();
        let x = input.mouse_pos.0/ dp.get_width() as f64 * 2.0 - 1.0;
        let y = 1.0 - input.mouse_pos.1/ dp.get_height() as f64 * 2.0;

        let inv_vp = (projection * self.get_view_matrix()).try_inverse().unwrap();

        let far_clip = Vector4::new(x, y, 1.0, 1.0);
        let far_clip_world = inv_vp * far_clip;
        let mouse_world_pos = far_clip_world.xyz() / far_clip_world.w;
        let dir = (mouse_world_pos - self.position).normalize();
        (self.position, dir)
    }

    #[inline]
    fn update_angles(&mut self, input: &Input) {
        if input.is_mouse_button_pressed(MouseButton::Button1) {
            let dx = input.d_mouse_pos.0;
            let dy = input.d_mouse_pos.1;
            if dx != 0.0 || dy != 0.0 {
                self.rotate_yaw_pitch(dx, dy);
                // let vec_dir = Vector3::new(-input.d_mouse_pos.1, input.d_mouse_pos.0, 0.0);
                // let norm = vec_dir.norm();
                // self.increase_rotation(&Unit::new_normalize(vec_dir), -norm * CAMERA_ROTATION_SPEED);
            }
        }
    }

    fn rotate_yaw_pitch(&mut self, dx: f64, dy: f64) {
        let yaw = dx * CAMERA_ROTATION_SPEED;
        let mut pitch = dy * CAMERA_ROTATION_SPEED;
        self.pitch += pitch;
        if self.pitch > PI/2. {
            self.pitch = PI/2.;
            pitch = 0.;
        }
        if self.pitch < -PI/2. {
            self.pitch = -PI/2.;
            pitch = 0.;
        }

        // Apply yaw in world space (pre‑multiply).
        self.quat = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -yaw) * self.quat;

        // Recompute camera right in updated orientation, then pitch around it.
        let right = self.quat.transform_vector(&Vector3::x_axis());
        let right = Unit::new_normalize(right);
        self.quat = UnitQuaternion::from_axis_angle(&right, pitch) * self.quat;

        println!("New rotation quaternion: {:?}", self.quat.euler_angles());
        // Keep unit length to avoid drift.
        self.quat.renormalize();
    }


    pub fn increase_rotation(&mut self, dir: &Unit<Vector3<f64>>, angle: f64) {
        let rot_quat = UnitQuaternion::from_axis_angle(dir, angle);
        println!("Rotating around axis {:?} by angle {:.4} radians", dir, angle);
        self.quat = rot_quat * self.quat;
        self.quat.renormalize();
        println!("New rotation quaternion: {:?}", self.quat.euler_angles());
    }

    #[inline]
    fn get_dp(&self, input: &Input) -> Vector3<f64> { // Use quaternion to remember previous rotation
        let mut front = self.quat.transform_vector(&Vector3::z_axis()).normalize();
        let mut right = self.quat.transform_vector(&Vector3::x_axis()).normalize();
        front.z = 0.;
        right.z = 0.;
        let front = front.normalize();
        let right = right.normalize();

        if input.is_key_pressed(Key::W) || input.is_key_pressed(Key::S) {
            let incr = if input.is_key_pressed(Key::W) { -1.0 } else { 1.0 };
            let dp = front * incr;
            dp
        } else if input.is_key_pressed(Key::A) || input.is_key_pressed(Key::D) {
            let incr = if input.is_key_pressed(Key::A) { -1.0 } else { 1.0 };
            let dp = right * incr;
            dp
        }else if input.is_key_pressed(Key::LeftShift) || input.is_key_pressed(Key::Space) {
            let incr = if input.is_key_pressed(Key::LeftShift) { 1.0 } else { -1.0 };
            let dp = Vector3::new(0., 0., 1.) * incr;
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