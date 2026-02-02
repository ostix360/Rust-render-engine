use nalgebra::{Matrix4, Vector4};
use crate::toolbox::opengl::shader::shader_program::{Shader, ShaderProgram};
use crate::toolbox::opengl::shader::uniform::matrix4uniform::Matrix4Uniform;
use crate::toolbox::opengl::shader::uniform::uniform::Uniform;
use crate::toolbox::opengl::shader::uniform::vec4uniform::Vec4Uniform;

pub struct ClassicShader {
    shader_program: ShaderProgram,
    projection_matrix: Matrix4Uniform,
    transformation_matrix: Matrix4Uniform,
    view_matrix: Matrix4Uniform,
    color: Vec4Uniform,
}

impl ClassicShader {
    pub fn new(program: ShaderProgram) -> ClassicShader {
        program.bind_attrib(0, "position");
        ClassicShader{
            shader_program: program,
            projection_matrix: Matrix4Uniform::new("projection_matrix"),
            transformation_matrix: Matrix4Uniform::new("transformation_matrix"),
            view_matrix: Matrix4Uniform::new("view_matrix"),
            color: Vec4Uniform::new("color"),
        }
    }

    pub fn load_projection_matrix(&self, matrix: Matrix4<f64>) {
        self.projection_matrix.load_matrix_to_uniform(matrix);
    }
    pub fn load_transformation_matrix(&self, matrix: Matrix4<f64>) {
        self.transformation_matrix.load_matrix_to_uniform(matrix);
    }
    pub fn load_view_matrix(&self, matrix: Matrix4<f64>) {
        self.view_matrix.load_matrix_to_uniform(matrix);
    }

    pub fn load_color(&self, color: Vector4<f64>) {
        self.color.load_vector_to_uniform(color);
    }
}

impl Shader for ClassicShader {
    fn bind(&self) {
        self.shader_program.bind()
    }

    fn unbind(&self) {
        self.shader_program.unbind()
    }

    fn store_all_uniforms(&mut self) {
        let mut uniforms: Box<[&mut Uniform]> = Box::new([
            &mut self.projection_matrix.uniform,
            &mut self.transformation_matrix.uniform,
            &mut self.view_matrix.uniform,
            &mut self.color.uniform,
        ]);
        self.shader_program.store_all_uniforms(&mut uniforms);
    }
}
