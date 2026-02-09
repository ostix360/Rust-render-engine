use crate::toolbox::color::Color;
use crate::toolbox::opengl::vao::VAO;
use nalgebra::{Matrix4, Rotation3, Translation3, Vector3, Vector4};

#[derive(PartialEq)]
pub struct Model<'a> {
    vao: &'a VAO,
    position: Vector3<f64>,
    rotation: Vector3<f64>,
    scale: f64,
    thickness: f64,
}

#[allow(dead_code)]
impl<'a> Model<'a> {
    pub fn new(vao: &'a VAO, position: Vector3<f64>, rotation: Vector3<f64>, scale: f64, thickness: f64) -> Self {
        Self {
            vao,
            position,
            rotation,
            scale,
            thickness,
        }
    }

    pub fn increase_rotation(&mut self, x: f64, y: f64, z: f64) {
        self.rotation.x += x;
        self.rotation.y += y;
        self.rotation.z += z;
    }

    pub fn get_vao(&self) -> &&VAO {
        &self.vao
    }

    pub fn get_vertex_count(&self) -> usize {
        self.vao.get_vertex_count()
    }

    pub fn get_transformation_matrix(&self, time: f64) -> Matrix4<f64> {
        let translation = Translation3::from(self.position);
        let rotation = Rotation3::from_euler_angles(self.rotation.x + time * 0.3, self.rotation.y, self.rotation.z);
        let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(self.scale, self.thickness, self.thickness));
        let result = translation.to_homogeneous() * rotation.to_homogeneous() * scale;
        result
    }
}

pub struct Sphere {
    pub position: Vector3<f64>,
    color: Color,
    size: f64,
}

impl Sphere {
    pub fn new(position: Vector3<f64>, color: Color, size: f64) -> Self {
        Self { position, color, size }
    }

    pub fn get_transformation_matrix(&self) -> Matrix4<f64> {
        let translation = Translation3::from(self.position);
        let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(self.size, self.size, self.size));
        translation.to_homogeneous() * scale
    }

    pub fn get_color(&self) -> Vector4<f64> {
        self.color.to_vector4()
    }
}

