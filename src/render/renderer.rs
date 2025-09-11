use std::ops::AddAssign;
use gl::{DrawElements, TRIANGLES, UNSIGNED_INT};
use gl::types::GLsizei;
use nalgebra::Matrix4;
use rustc_hash::FxHashMap;
use crate::graphics::model::Model;
use crate::render::classic_shader::ClassicShader;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::shader::shader_program::Shader;
use crate::toolbox::opengl::vao::VAO;

pub struct Renderer {
    shader: ClassicShader,
    time: f64,
}

impl Renderer {
    pub fn new(mut shader: ClassicShader, projection: Matrix4<f64>) -> Renderer {
        shader.store_all_uniforms();
        shader.bind();
        shader.load_projection_matrix(projection);
        shader.unbind();
        Renderer{
            shader,
            time: 0.0,
        }
    }
    
    pub fn render(&mut self, models: &FxHashMap<&VAO, Vec<&Model>>, cam: &Camera) {
        self.prepare(cam);
        self.time.add_assign(0.01);
        for vao in models.keys() {
            vao.binds(&[0, 1, 2]);
            if let Some(models) = models.get(vao) {
                for model in models {
                    self.shader.load_transformation_matrix(model.get_transformation_matrix(self.time));
                    unsafe {
                        DrawElements(TRIANGLES, vao.get_vertex_count() as GLsizei, UNSIGNED_INT, 0 as *const _);
                    }
                }
            }
            vao.unbinds(&[0, 1, 2])
        }
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