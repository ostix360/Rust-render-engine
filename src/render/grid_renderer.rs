use std::ptr::null;
use gl::types::GLsizei;
use nalgebra::Matrix4;
use crate::app::grid::Grid;
use crate::render::grid_shader::GridShader;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::shader::shader_program::Shader;

pub struct GridRenderer {
    shader: GridShader,
    projection: Matrix4<f64>,
}

impl GridRenderer {
    pub fn new(mut shader: GridShader, projection: Matrix4<f64>) -> GridRenderer {
        shader.store_all_uniforms();
        shader.bind();
        shader.load_projection_matrix(projection);
        shader.unbind();
        GridRenderer{
            shader,
            projection,
        }
    }

    pub fn render(&self, grid: &Grid, cam: &Camera) {
        self.prepare(cam);
        let data = grid.get_data();
        for (key, values) in data.iter(){
            key.get_vao().expect("You should create the vao before rendering the edge").binds(&[0]);
            for value in values {
                let transform = value.0;
                self.shader.load_transformation_matrix(transform);
                self.shader.load_color(value.1);
                unsafe {
                    gl::DrawElements(gl::LINES, (key.get_vao().unwrap().get_vertex_count()-1) as GLsizei, gl::UNSIGNED_INT, null())
                }
            }
            key.get_vao().unwrap().unbinds(&[0]);
        }
        self.unprepare();
    }

    pub fn update_shader_eqs(&mut self, new_eqs: [String; 3]){
        self.shader.edit_eqs(new_eqs);
        self.shader.bind();
        self.shader.load_projection_matrix(self.projection);
        self.shader.unbind();
    }

    fn prepare(&self, cam: &Camera) {
        self.shader.bind();
        self.shader.load_view_matrix(cam.get_view_matrix());
    }

    fn unprepare(&self) {
        self.shader.unbind();
    }
}