use std::ops::AddAssign;
use gl::{DrawArrays, DrawElements, Enable, TRIANGLES, UNSIGNED_INT};
use gl::types::GLsizei;
use nalgebra::Matrix4;
use rustc_hash::FxHashMap;
use crate::graphics::model::{Model, Sphere};
use crate::render::classic_shader::ClassicShader;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::open_gl_utils::open_gl_utils::set_wireframe_mode;
use crate::toolbox::opengl::shader::shader_program::Shader;
use crate::toolbox::opengl::vao::VAO;

pub struct Renderer {
    shader: ClassicShader,
    point_vao: VAO,
    time: f64,
}

impl Renderer {
    pub fn new(mut shader: ClassicShader, projection: Matrix4<f64>) -> Renderer {
        let point_vao = VAO::create_sphere();
        shader.store_all_uniforms();
        shader.bind();
        shader.load_projection_matrix(projection);
        shader.unbind();
        Renderer{
            shader,
            point_vao,
            time: 0.0,
        }
    }
    
    pub fn render(&mut self, models: &FxHashMap<&VAO, Vec<&Model>>, cam: &Camera) {
        self.prepare(cam);
        self.time.add_assign(0.01);
        set_wireframe_mode(true);
        for vao in models.keys() {
            vao.binds(&[0, 1, 2]);
            if let Some(models) = models.get(vao) {
                for model in models {
                    self.shader.load_transformation_matrix(model.get_transformation_matrix(0.));
                    unsafe {
                        DrawElements(TRIANGLES, vao.get_vertex_count() as GLsizei, UNSIGNED_INT, 0 as *const _);
                    }
                }
            }
            vao.unbinds(&[0, 1, 2])
        }
        self.finish();
    }
    
    pub fn draw_point(&mut self, points: Vec<Sphere>, cam: &Camera) {
        self.prepare(cam);

        self.point_vao.binds(&[0]);
        for point in points.iter() {
            self.shader.load_transformation_matrix(point.get_transformation_matrix());
            self.shader.load_color(point.get_color());
            unsafe {
                DrawElements(TRIANGLES, self.point_vao.get_vertex_count() as GLsizei, UNSIGNED_INT, 0 as *const _);
            }
        }
        self.point_vao.unbinds(&[0]);
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