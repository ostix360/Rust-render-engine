use std::ptr::null;
use gl::types::GLsizei;
use nalgebra::Matrix4;
use crate::graphics::model::RenderVField;
use crate::render::classic_shader::ClassicShader;
use crate::render::grid_shader::GridShader;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::shader::shader_program::Shader;
use crate::toolbox::opengl::vao::VAO;
use crate::toolbox::obj_loader::load_obj;

pub struct FieldRenderer {
    shader: GridShader,
    arrow_vao: VAO,
    projection: Matrix4<f64>,
}
impl FieldRenderer {
    pub fn new(mut shader: GridShader, projection: Matrix4<f64>) -> FieldRenderer {
        let arrow_vao = VAO::create_arrow();
        shader.bind();
        shader.store_all_uniforms();
        shader.load_projection_matrix(projection);
        shader.unbind();
        FieldRenderer {
            shader,
            arrow_vao,
            projection,
        }
    }
    pub fn render(&self, vectors: &[RenderVField], cam: &Camera) {
        self.prepare(cam);

        // We only bind position attribute (0) as ClassicShader only uses it
        self.arrow_vao.binds(&[0]);

        for vector_obj in vectors {
             let transform = vector_obj.get_transformation_matrix();

             self.shader.load_transformation_matrix(transform);
             self.shader.load_color(vector_obj.color);

             unsafe {
                 gl::DrawElements(gl::TRIANGLES, self.arrow_vao.get_vertex_count() as GLsizei, gl::UNSIGNED_INT, null());
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
