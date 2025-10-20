use nalgebra::Matrix4;
use crate::toolbox::opengl::shader::shader_program::{Shader, ShaderProgram};
use crate::toolbox::opengl::shader::uniform::matrix4uniform::Matrix4Uniform;

pub struct GridShader {
    shader_program: ShaderProgram,
    projection_matrix: Matrix4Uniform,
    transformation_matrix: Matrix4Uniform,
    view_matrix: Matrix4Uniform,
}

impl GridShader {
    pub fn new(program: ShaderProgram) -> GridShader {
        program.bind_attrib(0, "position");
        GridShader{
            shader_program: program,
            projection_matrix: Matrix4Uniform::new("projection_matrix"),
            transformation_matrix: Matrix4Uniform::new("transformation_matrix"),
            view_matrix: Matrix4Uniform::new("view_matrix"),
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
}

impl Shader for GridShader {
    fn bind(&self) {
        self.shader_program.bind()
    }

    fn unbind(&self) {
        self.shader_program.unbind()
    }

    fn store_all_uniforms(&mut self) {
        let mut uniforms: Box<[&mut crate::toolbox::opengl::shader::uniform::uniform::Uniform]> = Box::new([
            &mut self.projection_matrix.uniform,
            &mut self.transformation_matrix.uniform,
            &mut self.view_matrix.uniform,
        ]);
        self.shader_program.store_all_uniforms(&mut uniforms);
    }
}