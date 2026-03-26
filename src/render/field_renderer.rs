use crate::graphics::model::RenderVField;
use crate::render::field_shader::FieldShader;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::shader::shader_program::Shader;
use crate::toolbox::opengl::vao::VAO;
use gl::types::GLsizei;

pub struct FieldRenderer {
    shader: FieldShader,
    arrow_vao: VAO,
}
impl FieldRenderer {
    pub fn new(mut shader: FieldShader, projection: nalgebra::Matrix4<f64>) -> FieldRenderer {
        let arrow_vao = VAO::create_arrow();
        shader.bind();
        shader.store_all_uniforms();
        shader.load_projection_matrix(projection);
        shader.unbind();
        FieldRenderer { shader, arrow_vao }
    }
    pub fn render(&self, vectors: &[RenderVField], cam: &Camera) {
        self.prepare(cam);

        self.arrow_vao.binds(&[0]);

        for vector_obj in vectors {
            let transform = vector_obj.get_transformation_matrix();

            self.shader.load_transformation_matrix(transform);
            self.shader.load_color(vector_obj.color);

            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    self.arrow_vao.get_vertex_count() as GLsizei,
                    gl::UNSIGNED_INT,
                    0 as *const _,
                );
            }
        }

        self.arrow_vao.unbinds(&[0]);
        self.finish();
    }

    fn prepare(&self, cam: &Camera) {
        self.shader.bind();
        self.shader.load_view_matrix(cam.get_view_matrix());
    }
    fn finish(&self) {
        self.shader.unbind();
    }
}
