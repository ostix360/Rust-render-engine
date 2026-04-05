use crate::app::grid::Grid;
use crate::app::grid::SegmentDir;
use crate::render::grid_shader::GridShader;
use crate::render::master_render::SceneSpaceTransform;
use crate::toolbox::opengl::shader::shader_program::Shader;
use gl::types::GLsizei;
use nalgebra::Matrix4;
use std::ptr::null;

pub struct GridRenderer {
    shader: GridShader,
    projection: Matrix4<f64>, // Should use reference but lifetime needs to be handled
}

impl GridRenderer {
    pub fn new(mut shader: GridShader, projection: Matrix4<f64>) -> GridRenderer {
        shader.store_all_uniforms();
        shader.bind();
        shader.load_projection_matrix(&projection);
        shader.unbind();
        GridRenderer { shader, projection }
    }

    pub fn render(
        &self,
        grid: &Grid,
        view_matrix: &Matrix4<f64>,
        scene_transform: &SceneSpaceTransform,
    ) {
        self.prepare(view_matrix, scene_transform);
        let data = grid.get_data();
        for (key, values) in data.iter() {
            let vao = key
                .get_vao()
                .expect("You should create the vao before rendering the edge");
            let draw_count = vao.get_vertex_count() as GLsizei;
            vao.binds(&[0]);
            let mut last_dir: Option<SegmentDir> = None;
            for (transform, dir) in values {
                if last_dir != Some(*dir) {
                    self.shader.load_color_from_dir(*dir);
                    last_dir = Some(*dir);
                }
                self.shader.load_transformation_matrix(transform);
                unsafe { gl::DrawElements(gl::LINES, draw_count, gl::UNSIGNED_INT, null()) }
            }
            vao.unbinds(&[0]);
        }
        self.unprepare();
    }

    pub fn update_shader_eqs(&mut self, new_eqs: &[String; 3]) {
        self.shader.edit_eqs(new_eqs);
        self.shader.bind();
        self.shader.load_projection_matrix(&self.projection);
        self.shader.unbind();
    }

    pub fn update_projection(&mut self, projection: Matrix4<f64>) {
        self.projection = projection;
        self.shader.bind();
        self.shader.load_projection_matrix(&self.projection);
        self.shader.unbind();
    }

    fn prepare(&self, view_matrix: &Matrix4<f64>, scene_transform: &SceneSpaceTransform) {
        self.shader.bind();
        self.shader.load_view_matrix(view_matrix);
        self.shader.load_scene_transform(scene_transform);
    }

    fn unprepare(&self) {
        self.shader.unbind();
    }
}
