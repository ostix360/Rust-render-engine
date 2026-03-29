use crate::app::coords_sys::CoordsSys;
use crate::app::grid::{Grid, GridConfig};
use crate::app::grid_world::GridWorld;
use crate::app::ui::{GridUiState, SpacialEqs};
use crate::graphics::model::{RenderVField, Sphere};
use crate::maths::differential::Form;
use crate::maths::field::VectorField;
use crate::maths::Point;
use crate::render::master_render::MasterRenderer;
use crate::toolbox::camera::Camera;
use crate::toolbox::color::WHITE;
use crate::toolbox::opengl::display_manager::DisplayManager;
use mathhook_core::formatter::simple::SimpleContext;
use mathhook_core::SimpleFormatter;
use nalgebra::{vector, Matrix4, Vector3, Vector4};
use std::sync::{Arc, Mutex};
use typed_floats::NonNaN;

#[derive(Clone, PartialEq)]
struct AppliedConfig {
    grid_config: GridConfig,
    coord_eqs: [String; 3],
    field_eqs: [String; 3],
    normalize_field: bool,
}

impl AppliedConfig {
    fn from_ui(state: &GridUiState) -> Self {
        let context = SimpleContext::default();
        Self {
            grid_config: state.to_grid_config(),
            coord_eqs: [
                state.coords_sys.x.eq
                    .to_simple(&context)
                    .expect("Error while converting x eq"),
                state.coords_sys.y.eq
                    .to_simple(&context)
                    .expect("Error while converting y eq"),
                state.coords_sys.z.eq
                    .to_simple(&context)
                    .expect("Error while converting z eq"),
            ],
            field_eqs: [
                state.field.x.eq_str.clone(),
                state.field.y.eq_str.clone(),
                state.field.z.eq_str.clone(),
            ],
            normalize_field: state.normalize_field,
        }
    }

    fn diff(&self, next: &Self) -> ApplyDiff {
        ApplyDiff {
            grid_changed: self.grid_config != next.grid_config,
            coords_changed: self.coord_eqs != next.coord_eqs,
            field_changed: self.field_eqs != next.field_eqs,
            normalize_changed: self.normalize_field != next.normalize_field,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ApplyDiff {
    grid_changed: bool,
    coords_changed: bool,
    field_changed: bool,
    normalize_changed: bool,
}

impl ApplyDiff {
    fn geometry_changed(self) -> bool {
        self.grid_changed || self.coords_changed
    }

    fn field_cache_changed(self) -> bool {
        self.geometry_changed() || self.field_changed
    }

    fn render_field_changed(self) -> bool {
        self.field_cache_changed() || self.normalize_changed
    }
}

#[derive(Clone)]
struct FieldSample {
    abstract_pos: Vector3<f64>,
    world_pos: Vector3<f64>,
    basis: [Vector3<f64>; 3],
}

impl FieldSample {
    fn vector_to_world(&self, vector: Vector3<f64>) -> Vector3<f64> {
        self.basis[0] * vector.x + self.basis[1] * vector.y + self.basis[2] * vector.z
    }
}

pub struct World {
    field: VectorField,
    render_field: Vec<RenderVField>,
    field_samples: Vec<FieldSample>,
    cached_field_vectors: Vec<Vector3<f64>>,
    normalize_field: bool,
    renderer: MasterRenderer,
    grid: Grid,
    grid_world: GridWorld,
    shared_ui_state: Arc<Mutex<GridUiState>>,
    last_counter: u64,
    sphere: Option<Sphere>,
    applied_config: AppliedConfig,
}

impl World {
    pub fn new(shared_ui_state: Arc<Mutex<GridUiState>>, display_manager: &DisplayManager) -> Self {
        let initial_state = shared_ui_state.lock().unwrap().clone();
        let applied_config = AppliedConfig::from_ui(&initial_state);
        let (grid, field, field_samples, grid_points) = Self::init(initial_state.clone());
        let mut world = Self {
            field,
            render_field: Vec::new(),
            field_samples,
            cached_field_vectors: Vec::new(),
            normalize_field: initial_state.normalize_field,
            renderer: MasterRenderer::new(
                display_manager.get_width() as f64,
                display_manager.get_height() as f64,
            ),
            grid,
            grid_world: GridWorld::from_points(grid_points),
            shared_ui_state,
            last_counter: 0,
            sphere: None,
            applied_config,
        };
        world.recompute_cached_field_vectors();
        world.rebuild_render_field();
        world
    }

    fn init(initial_state: GridUiState) -> (Grid, VectorField, Vec<FieldSample>, Vec<[f64; 3]>) {
        let config = initial_state.to_grid_config();
        let x_eq = initial_state.coords_sys.x.eq;
        let y_eq = initial_state.coords_sys.y.eq;
        let z_eq = initial_state.coords_sys.z.eq;
        let sys_coord = CoordsSys::new(x_eq, y_eq, z_eq);
        let mut grid = Grid::new(sys_coord);
        grid.update_config(&config);
        let (field_samples, grid_points) = Self::build_grid_cache(&grid);
        let vf = Self::build_vector_field(&initial_state.field, &grid);
        (grid, vf, field_samples, grid_points)
    }

    fn build_vector_field(field: &SpacialEqs, grid: &Grid) -> VectorField {
        let field_eqs = vec![field.x.eq.clone(), field.y.eq.clone(), field.z.eq.clone()];
        VectorField::from_otn(Form::new(field_eqs, 1), grid.get_coords().get_space())
    }

    fn build_grid_cache(grid: &Grid) -> (Vec<FieldSample>, Vec<[f64; 3]>) {
        let data = grid.get_data();
        let coords = grid.get_coords();

        let mut field_sample_capacity = 0usize;
        let mut grid_point_capacity = 0usize;
        for (edge, transforms) in data.iter() {
            field_sample_capacity += transforms.len();
            grid_point_capacity += edge.get_nb_vertices() * transforms.len();
        }

        let mut field_samples = Vec::with_capacity(field_sample_capacity);
        let mut grid_points = Vec::with_capacity(grid_point_capacity);

        for (edge, transforms) in data.iter() {
            let vertices = edge.get_vertices();
            if vertices.is_empty() {
                continue;
            }

            for (transform, _) in transforms.iter() {
                let abstract_pos = Self::transform_vertex(transform, &vertices[0]);

                // Cache the geometry-dependent part of the field sampling. Field-only applies
                // can reuse these positions and basis vectors without touching grid generation.
                field_samples.push(FieldSample {
                    abstract_pos,
                    world_pos: coords.eval_position(abstract_pos),
                    basis: coords.eval_tangent_basis(abstract_pos),
                });

                for vertex in vertices.iter() {
                    let abstract_vertex = Self::transform_vertex(transform, vertex);
                    let world_vertex = coords.eval_position(abstract_vertex);
                    grid_points.push([world_vertex.x, world_vertex.y, world_vertex.z]);
                }
            }
        }

        (field_samples, grid_points)
    }

    fn transform_vertex(transform: &Matrix4<f64>, vertex: &Vector3<NonNaN<f64>>) -> Vector3<f64> {
        let vec4 = Vector4::new(vertex.x.get(), vertex.y.get(), vertex.z.get(), 1.0);
        (transform * vec4).xyz()
    }

    fn apply_state(&mut self, state: GridUiState) {
        let next_config = AppliedConfig::from_ui(&state);
        let diff = self.applied_config.diff(&next_config);

        // Each branch invalidates one cache boundary. That keeps the apply pipeline readable:
        // geometry rebuilds the sampled grid cache, field changes rebuild sampled vectors, and
        // normalization only rebuilds the final render transforms.
        if diff.coords_changed {
            let coords = state.coords_sys.clone();
            let coord_sys = CoordsSys::new(coords.x.eq, coords.y.eq, coords.z.eq);
            self.grid.set_coordinates(coord_sys);
            self.renderer
                .grid_renderer
                .update_shader_eqs(&next_config.coord_eqs);
        }

        if diff.geometry_changed() {
            self.grid.update_config(&next_config.grid_config);
            let (field_samples, grid_points) = Self::build_grid_cache(&self.grid);
            self.field_samples = field_samples;
            self.grid_world.replace_points(grid_points);
        }

        if diff.coords_changed || diff.field_changed {
            self.field = Self::build_vector_field(&state.field, &self.grid);
        }

        self.normalize_field = next_config.normalize_field;

        if diff.field_cache_changed() {
            self.recompute_cached_field_vectors();
        }

        if diff.render_field_changed() {
            self.rebuild_render_field();
        }

        self.applied_config = next_config;
        self.last_counter = state.apply_counter;
    }

    pub fn update(&mut self, mouse_info: (Vector3<f64>, Vector3<f64>)) {
        let mut pending_state = None;
        {
            let sharded = self.shared_ui_state.lock().unwrap();
            if self.last_counter != sharded.apply_counter {
                pending_state = Some(sharded.clone());
            }
        }

        if let Some(state) = pending_state {
            self.apply_state(state);
        }

        let nearest_point = self
            .grid_world
            .ray_cast(&mouse_info.0, &mouse_info.1, 0.45, 200.);
        if let Some(point) = nearest_point {
            // println!(
            //     "Nearest point at: x: {}, y: {}, z: {}",
            //     point.0, point.1, point.2
            // );
            self.sphere = Some(Sphere::new(vector![point.0, point.1, point.2], WHITE, 0.1));
        } else {
            self.sphere = None;
        }
    }

    pub fn render(&self, camera: &Camera) {
        self.renderer
            .render(&self.grid, &self.render_field, camera, &self.sphere)
    }

    fn recompute_cached_field_vectors(&mut self) {
        self.cached_field_vectors.clear();
        self.cached_field_vectors.reserve(self.field_samples.len());

        for sample in &self.field_samples {
            let vec_res = self.field.at(Point {
                x: sample.abstract_pos.x,
                y: sample.abstract_pos.y,
                z: sample.abstract_pos.z,
            });
            self.cached_field_vectors
                .push(sample.vector_to_world(Vector3::new(vec_res.x, vec_res.y, vec_res.z)));
        }
    }

    fn rebuild_render_field(&mut self) {
        self.render_field.clear();
        self.render_field.reserve(self.field_samples.len());

        for (sample, world_vector) in self
            .field_samples
            .iter()
            .zip(self.cached_field_vectors.iter().copied())
        {
            let mut render_vector = world_vector;
            if self.normalize_field {
                let magnitude = render_vector.norm();
                if magnitude > 1e-6 {
                    render_vector /= magnitude;
                }
            }

            self.render_field.push(RenderVField::new(
                sample.world_pos,
                render_vector,
                Vector4::new(1.0, 1.0, 0.0, 1.0),
            ));
        }
    }

    pub fn get_projection(&self) -> Matrix4<f64> {
        self.renderer.projection
    }
}

#[cfg(test)]
mod tests {
    use super::AppliedConfig;
    use crate::app::ui::GridUiState;

    #[test]
    fn apply_diff_is_scoped_to_field_changes() {
        let current = AppliedConfig::from_ui(&GridUiState::default());
        let mut next_state = GridUiState::default();
        next_state.field.x.eq_str = "2".to_string();
        let next = AppliedConfig::from_ui(&next_state);

        let diff = current.diff(&next);

        assert!(diff.field_changed);
        assert!(!diff.grid_changed);
        assert!(!diff.coords_changed);
        assert!(!diff.normalize_changed);
    }

    #[test]
    fn apply_diff_is_scoped_to_grid_changes() {
        let current = AppliedConfig::from_ui(&GridUiState::default());
        let mut next_state = GridUiState::default();
        next_state.nb_x = 9.0;
        let next = AppliedConfig::from_ui(&next_state);

        let diff = current.diff(&next);

        assert!(diff.grid_changed);
        assert!(!diff.field_changed);
        assert!(!diff.coords_changed);
        assert!(!diff.normalize_changed);
    }
}
