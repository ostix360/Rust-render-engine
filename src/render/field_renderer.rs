use crate::graphics::model::RenderVField;
use crate::render::classic_shader::ClassicShader;
use crate::render::grid_shader::GridShader;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::shader::shader_program::Shader;
use crate::toolbox::opengl::vao::VAO;
use gl::types::GLsizei;
use nalgebra::Matrix4;
use std::ptr::null;

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
        FieldRenderer { shader, arrow_vao, projection }
    }
    pub fn render(&self, vectors: &[RenderVField], cam: &Camera) {
        self.prepare(cam);

        // We only bind position attribute (0) as ClassicShader only uses it
        self.arrow_vao.binds(&[0]);

        for vector_obj in vectors {
            let transform = vector_obj.get_transformation_matrix();

            self.shader.load_transformation_matrix(transform);
            self.shader.load_color(vector_obj.color.xyz());

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

    pub fn update_shader_eqs(&mut self, new_eqs: &[String; 3]) {
        self.shader.edit_eqs(new_eqs);
        self.shader.bind();
        self.shader.load_projection_matrix(self.projection);
        self.shader.unbind();
    }


    fn prepare(&self, cam: &Camera) {
        self.shader.bind();
        self.shader.load_view_matrix(cam.get_view_matrix());
    }
    fn finish(&self) {
        self.shader.unbind();
    }
}
