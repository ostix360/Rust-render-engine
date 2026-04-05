use crate::graphics::model::RenderVField;
use crate::render::field_shader::FieldShader;
use crate::toolbox::opengl::shader::shader_program::Shader;
use crate::toolbox::opengl::vao::VAO;
use gl::types::GLsizei;
use nalgebra::{Matrix4, Vector4};

pub struct FieldRenderer {
    shader: FieldShader,
    arrow_vao: VAO,
}
impl FieldRenderer {
    pub fn new(mut shader: FieldShader, projection: &Matrix4<f64>) -> FieldRenderer {
        let arrow_vao = VAO::create_arrow();
        shader.bind();
        shader.store_all_uniforms();
        shader.load_projection_matrix(projection);
        shader.unbind();
        FieldRenderer { shader, arrow_vao }
    }
    pub fn render(&self, vectors: &[RenderVField], view_matrix: &Matrix4<f64>) {
        self.prepare(view_matrix);

        let draw_count = self.arrow_vao.get_vertex_count() as GLsizei;
        self.arrow_vao.binds(&[0]);
        let mut last_color: Option<Vector4<f64>> = None;

        for vector_obj in vectors {
            if !vector_obj.is_renderable() {
                continue;
            }

            if last_color.as_ref() != Some(&vector_obj.color) {
                self.shader.load_color(vector_obj.color.clone());
                last_color = Some(vector_obj.color.clone());
            }
            self.shader
                .load_transformation_matrix(vector_obj.get_transformation_matrix());

            unsafe {
                gl::DrawElements(gl::TRIANGLES, draw_count, gl::UNSIGNED_INT, 0 as *const _);
            }
        }

        self.arrow_vao.unbinds(&[0]);
        self.finish();
    }

    pub fn update_projection(&mut self, projection: &Matrix4<f64>) {
        self.shader.bind();
        self.shader.load_projection_matrix(projection);
        self.shader.unbind();
    }

    fn prepare(&self, view_matrix: &Matrix4<f64>) {
        self.shader.bind();
        self.shader.load_view_matrix(view_matrix);
    }
    fn finish(&self) {
        self.shader.unbind();
    }
}
