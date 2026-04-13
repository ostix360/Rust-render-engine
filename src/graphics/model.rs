//! Renderable scene primitives and transformation helpers.

use crate::toolbox::color::Color;
use crate::toolbox::opengl::vao::VAO;
use nalgebra::{Matrix4, Rotation3, Translation3, UnitQuaternion, Vector3, Vector4};

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
    /// Creates a new `Model`.
    pub fn new(
        vao: &'a VAO,
        position: Vector3<f64>,
        rotation: Vector3<f64>,
        scale: f64,
        thickness: f64,
    ) -> Self {
        Self {
            vao,
            position,
            rotation,
            scale,
            thickness,
        }
    }

    /// Applies an incremental change to the rotation.
    pub fn increase_rotation(&mut self, x: f64, y: f64, z: f64) {
        self.rotation.x += x;
        self.rotation.y += y;
        self.rotation.z += z;
    }

    /// Returns the current vao.
    pub fn get_vao(&self) -> &&VAO {
        &self.vao
    }

    /// Returns the current vertex count.
    pub fn get_vertex_count(&self) -> usize {
        self.vao.get_vertex_count()
    }

    /// Returns the current transformation matrix.
    pub fn get_transformation_matrix(&self, time: f64) -> Matrix4<f64> {
        let translation = Translation3::from(self.position);
        let rotation = Rotation3::from_euler_angles(
            self.rotation.x + time * 0.3,
            self.rotation.y,
            self.rotation.z,
        );
        let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(
            self.scale,
            self.thickness,
            self.thickness,
        ));
        let result = translation.to_homogeneous() * rotation.to_homogeneous() * scale;
        result
    }
}

pub struct Sphere {
    #[allow(dead_code)]
    pub position: Vector3<f64>,
    color: Color,
    transformation: Matrix4<f64>,
}

impl Sphere {
    /// Creates a new `Sphere`.
    pub fn new(position: Vector3<f64>, color: Color, size: f64) -> Self {
        let translation = Translation3::from(position);
        let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(size, size, size));
        let transformation = translation.to_homogeneous() * scale;
        Self {
            position,
            color,
            transformation,
        }
    }

    /// Constructs `Sphere` from rgba.
    pub fn from_rgba(position: Vector3<f64>, rgba: Vector4<f64>, size: f64) -> Self {
        Self::new(
            position,
            Color::new(rgba.x as f32, rgba.y as f32, rgba.z as f32, rgba.w as f32),
            size,
        )
    }

    /// Returns the current transformation matrix.
    pub fn get_transformation_matrix(&self) -> &Matrix4<f64> {
        &self.transformation
    }

    /// Returns the current color.
    pub fn get_color(&self) -> Vector4<f64> {
        self.color.to_vector4()
    }
}

#[allow(dead_code)]
pub struct UnitArrow {
    pub position: Vector3<f64>,
    pub direction: Vector3<f64>,
    pub color: Color,
}

#[allow(dead_code)]
impl UnitArrow {
    /// Creates a new `UnitArrow`.
    pub fn new(position: Vector3<f64>, direction: Vector3<f64>, color: Color) -> Self {
        Self {
            position,
            direction,
            color,
        }
    }

    /// Returns the current transformation matrix.
    pub fn get_transformation_matrix(&self) -> Matrix4<f64> {
        let translation = Translation3::from(self.position);
        translation.to_homogeneous()
            * Rotation3::face_towards(&self.direction, &Vector3::y()).to_homogeneous()
    }

    /// Returns the current color.
    pub fn get_color(&self) -> Vector4<f64> {
        self.color.to_vector4()
    }
}

pub struct RenderVField {
    pub color: Vector4<f64>,
    transform: Matrix4<f64>,
    renderable: bool,
}

impl RenderVField {
    /// Creates a new `RenderVField`.
    pub fn new(position: Vector3<f64>, vector: Vector3<f64>, color: Vector4<f64>) -> Self {
        let (transform, renderable) = Self::build_transform(&position, &vector);
        Self {
            color,
            transform,
            renderable,
        }
    }

    /// Builds the transform that renders one vector-field arrow.
    ///
    /// Zero-length vectors produce a non-renderable marker, while non-zero vectors are oriented
    /// and scaled to match their magnitude.
    fn build_transform(position: &Vector3<f64>, vector: &Vector3<f64>) -> (Matrix4<f64>, bool) {
        const ARROW_SCALE: f64 = 0.02;

        let magnitude = vector.norm();
        if magnitude < 1e-6 {
            return (Matrix4::zeros(), false);
        }

        let radius_scale = ARROW_SCALE;
        let length_scale = magnitude * ARROW_SCALE;
        let target_dir = vector.normalize();
        let up = Vector3::y_axis();

        let rotation = UnitQuaternion::rotation_between(&up, &target_dir)
            .unwrap_or(UnitQuaternion::identity());
        let translation = Translation3::from(*position);

        let mut transform = nalgebra::Isometry3::from_parts(translation, rotation).to_homogeneous();
        transform.prepend_nonuniform_scaling_mut(&Vector3::new(
            radius_scale,
            10.0 * length_scale,
            radius_scale,
        ));
        (transform, true)
    }

    /// Returns the current transformation matrix.
    pub fn get_transformation_matrix(&self) -> &Matrix4<f64> {
        &self.transform
    }

    /// Returns whether renderable.
    pub fn is_renderable(&self) -> bool {
        self.renderable
    }
}
