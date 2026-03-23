use crate::app::grid::Grid;
use crate::graphics::model::{RenderVField, Sphere};
use crate::render;
use crate::render::grid_renderer::GridRenderer;
use crate::render::grid_shader::GridShader;
use crate::render::renderer::Renderer;
use crate::render::field_renderer::FieldRenderer;
use crate::maths::field::VectorField;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::open_gl_utils::open_gl_utils::clear_gl;
use crate::toolbox::opengl::shader::shader_program::ShaderProgram;
use nalgebra::{Matrix4, Perspective3};

const NEAR: f64 = 0.01;
const FAR: f64 = 750.0;

pub struct MasterRenderer {
    pub grid_renderer: GridRenderer,
    pub renderer: Renderer,
    pub field_renderer: FieldRenderer,
    pub projection: Matrix4<f64>,
}

impl MasterRenderer {
    pub fn new(w: f64, h: f64) -> Self {
        let (grid_renderer, renderer, field_renderer, projection) = Self::init(w, h);
        Self {
            grid_renderer,
            renderer,
            field_renderer,
            projection,
        }
    }

    fn init(w: f64, h: f64) -> (GridRenderer, Renderer, FieldRenderer, Matrix4<f64>) {
        let aspect_ratio = w / h;
        let projection = Perspective3::new(aspect_ratio, 1.6, NEAR, FAR);
        let grid_shader_prog = ShaderProgram::new("grid");
        let grid_shader = GridShader::new(grid_shader_prog);
        let grid_renderer = GridRenderer::new(grid_shader, projection.to_homogeneous());
        let classic_shader_prog = ShaderProgram::new("classic");
        let classic_shader = render::classic_shader::ClassicShader::new(classic_shader_prog);
        let point_renderer = Renderer::new(classic_shader, projection.to_homogeneous());

        let field_shader_prog = ShaderProgram::new("grid");
        let field_shader = GridShader::new(field_shader_prog);
        let field_renderer = FieldRenderer::new(field_shader, projection.to_homogeneous());

        (grid_renderer, point_renderer, field_renderer, projection.to_homogeneous())
    }

    pub fn render(&self, grid: &Grid, field_vectors: &[Vec<RenderVField>], camera: &Camera, sphere: &Option<Sphere>) {
        clear_gl();
        self.grid_renderer.render(&grid, &camera);
        // for vectors in field_vectors {
        //     self.field_renderer.render(vectors, camera);
        // }
        if let Some(sphere) = sphere {
            self.renderer.draw_point(sphere, &camera);
        }
    }
}
