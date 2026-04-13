//! Top-level render orchestration for grid, field, and overlay passes.

use crate::app::grid::Grid;
use crate::app::tangent_space::SceneSpaceTransform;
use crate::graphics::model::{RenderVField, Sphere};
use crate::render::classic_shader::ClassicShader;
use crate::render::field_renderer::FieldRenderer;
use crate::render::field_shader::FieldShader;
use crate::render::grid_renderer::GridRenderer;
use crate::render::grid_shader::GridShader;
use crate::render::renderer::Renderer;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::open_gl_utils::open_gl_utils::clear_gl;
use crate::toolbox::opengl::shader::shader_program::ShaderProgram;
use nalgebra::{Matrix4, Perspective3};

const NEAR: f64 = 0.01;
const FAR: f64 = 750.0;
const BASE_FOVY: f64 = 1.6;
const TANGENT_FOVY: f64 = 0.72;

pub struct MasterRenderer {
    pub grid_renderer: GridRenderer,
    pub renderer: Renderer,
    pub field_renderer: FieldRenderer,
    pub projection: Matrix4<f64>,
    aspect_ratio: f64,
}

impl MasterRenderer {
    /// Creates the renderer passes and initial projection matrix for the given viewport size.
    pub fn new(w: f64, h: f64) -> Self {
        let aspect_ratio = aspect_ratio_for(w, h);
        let (grid_renderer, renderer, field_renderer, projection) = Self::init(aspect_ratio);
        Self {
            grid_renderer,
            renderer,
            field_renderer,
            projection,
            aspect_ratio,
        }
    }

    /// Builds the grid, classic, and field renderer passes that share the same projection
    /// matrix.
    fn init(aspect_ratio: f64) -> (GridRenderer, Renderer, FieldRenderer, Matrix4<f64>) {
        let projection = projection_for_zoom_mix(aspect_ratio, 0.0);
        let grid_shader_prog = ShaderProgram::new("grid");
        let grid_shader = GridShader::new(grid_shader_prog);
        let grid_renderer = GridRenderer::new(grid_shader, projection.clone());
        let classic_shader_prog = ShaderProgram::new("classic");
        let classic_shader = ClassicShader::new(classic_shader_prog);
        let point_renderer = Renderer::new(classic_shader, &projection);

        let field_shader_prog = ShaderProgram::new("field");
        let field_shader = FieldShader::new(field_shader_prog);
        let field_renderer = FieldRenderer::new(field_shader, &projection);

        (grid_renderer, point_renderer, field_renderer, projection)
    }

    /// Updates the projection matrix used by all render passes for the current tangent zoom
    /// blend.
    pub fn set_zoom_mix(&mut self, mix: f64) {
        self.projection = projection_for_zoom_mix(self.aspect_ratio, mix);
        self.grid_renderer
            .update_projection(self.projection.clone());
        self.renderer.update_projection(&self.projection);
        self.field_renderer.update_projection(&self.projection);
    }

    /// Runs the frame render passes in the correct order for the current scene state.
    pub fn render(
        &self,
        grid: &Grid,
        field_vectors: &[RenderVField],
        form_samples: &[Sphere],
        show_grid: bool,
        show_vector_field: bool,
        camera: &Camera,
        sphere: &Option<Sphere>,
        scene_transform: &SceneSpaceTransform,
    ) {
        clear_gl();
        let view_matrix = camera.get_view_matrix();
        if show_grid {
            self.grid_renderer
                .render(&grid, &view_matrix, scene_transform);
        }
        if show_vector_field {
            self.field_renderer.render(field_vectors, &view_matrix);
        } else {
            self.renderer.draw_points(form_samples, &view_matrix);
        }
        if let Some(sphere) = sphere {
            self.renderer.draw_point(sphere, &view_matrix);
        }
    }
}

/// Computes a safe aspect ratio from the viewport size.
///
/// The height denominator is clamped away from zero.
fn aspect_ratio_for(w: f64, h: f64) -> f64 {
    w / h.max(1.0)
}

/// Builds the perspective projection matrix for the current tangent zoom blend.
fn projection_for_zoom_mix(aspect_ratio: f64, zoom_mix: f64) -> Matrix4<f64> {
    Perspective3::new(aspect_ratio, fovy_for_zoom_mix(zoom_mix), NEAR, FAR).to_homogeneous()
}

/// Interpolates the vertical field of view between the world and tangent settings.
fn fovy_for_zoom_mix(zoom_mix: f64) -> f64 {
    let clamped_mix = zoom_mix.clamp(0.0, 1.0);
    BASE_FOVY + (TANGENT_FOVY - BASE_FOVY) * clamped_mix
}

#[cfg(test)]
mod tests {
    use super::projection_for_zoom_mix;

    #[test]
    fn tangent_zoom_tightens_projection() {
        let world_projection = projection_for_zoom_mix(16.0 / 9.0, 0.0);
        let tangent_projection = projection_for_zoom_mix(16.0 / 9.0, 1.0);

        assert!(tangent_projection[(1, 1)] > world_projection[(1, 1)]);
    }
}
