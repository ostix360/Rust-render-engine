use crate::app::coords_sys::CoordsSys;
use crate::app::grid::{Grid, GridConfig};
use crate::app::grid_world::{GridSample, GridWorld};
use crate::app::tangent_space::{SceneSpaceTransform, TangentSpace};
use crate::app::ui::{GridUiState, SpacialEqs};
use crate::graphics::model::{RenderVField, Sphere};
use crate::maths::differential::Form;
use crate::maths::field::VectorField;
use crate::maths::Point;
use crate::render::master_render::MasterRenderer;
use crate::toolbox::camera::Camera;
use crate::toolbox::color::WHITE;
use crate::toolbox::input::Input;
use crate::toolbox::opengl::display_manager::DisplayManager;
use mathhook_core::formatter::simple::SimpleContext;
use mathhook_core::SimpleFormatter;
use nalgebra::{vector, Matrix4, Vector3, Vector4};
use std::sync::{Arc, Mutex};
use typed_floats::NonNaN;

const SPHERE_SIZE: f64 = 0.1;
const FORM_SAMPLE_BASE_SIZE: f64 = 0.03;
const FORM_SAMPLE_SIZE_SCALE: f64 = 0.08;

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
    render_form_samples: Vec<Sphere>,
    field_samples: Vec<FieldSample>,
    cached_field_components: Vec<Vector3<f64>>,
    cached_dual_components: Vec<Vector3<f64>>,
    cached_field_vectors: Vec<Vector3<f64>>,
    normalize_field: bool,
    renderer: MasterRenderer,
    grid: Grid,
    grid_world: GridWorld,
    shared_ui_state: Arc<Mutex<GridUiState>>,
    deferred_apply_state: Option<GridUiState>,
    last_counter: u64,
    sphere: Option<Sphere>,
    tangent_space: TangentSpace,
    applied_config: AppliedConfig,
}

impl World {
    pub fn new(shared_ui_state: Arc<Mutex<GridUiState>>, display_manager: &DisplayManager) -> Self {
        let initial_state = shared_ui_state.lock().unwrap().clone();
        let applied_config = AppliedConfig::from_ui(&initial_state);
        let (grid, field, field_samples, grid_samples) = Self::init(initial_state.clone());
        let mut world = Self {
            field,
            render_field: Vec::new(),
            render_form_samples: Vec::new(),
            field_samples,
            cached_field_components: Vec::new(),
            cached_dual_components: Vec::new(),
            cached_field_vectors: Vec::new(),
            normalize_field: initial_state.normalize_field,
            renderer: MasterRenderer::new(
                display_manager.get_width() as f64,
                display_manager.get_height() as f64,
            ),
            grid,
            grid_world: GridWorld::from_samples(grid_samples),
            shared_ui_state,
            deferred_apply_state: None,
            last_counter: 0,
            sphere: None,
            tangent_space: TangentSpace::new(),
            applied_config,
        };
        world.recompute_cached_field_vectors();
        world.rebuild_render_field();
        world
    }

    fn init(initial_state: GridUiState) -> (Grid, VectorField, Vec<FieldSample>, Vec<GridSample>) {
        let config = initial_state.to_grid_config();
        let x_eq = initial_state.coords_sys.x.eq;
        let y_eq = initial_state.coords_sys.y.eq;
        let z_eq = initial_state.coords_sys.z.eq;
        let sys_coord = CoordsSys::new(x_eq, y_eq, z_eq);
        let mut grid = Grid::new(sys_coord);
        grid.update_config(&config);
        let (field_samples, grid_samples) = Self::build_grid_cache(&grid);
        let vf = Self::build_vector_field(&initial_state.field, &grid);
        (grid, vf, field_samples, grid_samples)
    }

    fn build_vector_field(field: &SpacialEqs, grid: &Grid) -> VectorField {
        let field_eqs = vec![field.x.eq.clone(), field.y.eq.clone(), field.z.eq.clone()];
        VectorField::from_otn(Form::new(field_eqs, 1), grid.get_coords().get_space())
    }

    fn build_grid_cache(grid: &Grid) -> (Vec<FieldSample>, Vec<GridSample>) {
        let data = grid.get_data();
        let coords = grid.get_coords();

        let mut field_sample_capacity = 0usize;
        let mut grid_point_capacity = 0usize;
        for (edge, transforms) in data.iter() {
            field_sample_capacity += transforms.len();
            grid_point_capacity += edge.get_nb_vertices() * transforms.len();
        }

        let mut field_samples = Vec::with_capacity(field_sample_capacity);
        let mut grid_samples = Vec::with_capacity(grid_point_capacity);

        for (edge, transforms) in data.iter() {
            let vertices = edge.get_vertices();
            if vertices.is_empty() {
                continue;
            }

            for (transform, _) in transforms.iter() {
                let abstract_pos = Self::transform_vertex(transform, &vertices[0]);

                field_samples.push(FieldSample {
                    abstract_pos,
                    world_pos: coords.eval_position(abstract_pos),
                    basis: coords.eval_tangent_basis(abstract_pos),
                });

                for vertex in vertices.iter() {
                    let abstract_vertex = Self::transform_vertex(transform, vertex);
                    let world_vertex = coords.eval_position(abstract_vertex);
                    grid_samples.push(GridSample {
                        world_pos: world_vertex,
                        abstract_pos: abstract_vertex,
                    });
                }
            }
        }

        (field_samples, grid_samples)
    }

    fn transform_vertex(transform: &Matrix4<f64>, vertex: &Vector3<NonNaN<f64>>) -> Vector3<f64> {
        let vec4 = Vector4::new(vertex.x.get(), vertex.y.get(), vertex.z.get(), 1.0);
        (transform * vec4).xyz()
    }

    fn apply_state(&mut self, state: GridUiState, next_config: AppliedConfig, diff: ApplyDiff) {
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
            let (field_samples, grid_samples) = Self::build_grid_cache(&self.grid);
            self.field_samples = field_samples;
            self.grid_world.replace_samples(grid_samples);
        }

        if diff.coords_changed || diff.field_changed {
            self.field = Self::build_vector_field(&state.field, &self.grid);
        }

        self.normalize_field = next_config.normalize_field;

        if diff.field_cache_changed() {
            self.recompute_cached_field_vectors();
        }

        self.applied_config = next_config;
        self.last_counter = state.apply_counter;
    }

    pub fn update(
        &mut self,
        input: &Input,
        dt: f64,
        display_manager: &DisplayManager,
        camera: &mut Camera,
    ) {
        let mut pending_state = self.deferred_apply_state.take();
        {
            let shared = self.shared_ui_state.lock().unwrap();
            if pending_state.is_none() && self.last_counter != shared.apply_counter {
                pending_state = Some(shared.clone());
            }
        }

        if let Some(state) = pending_state {
            let next_config = AppliedConfig::from_ui(&state);
            let diff = self.applied_config.diff(&next_config);
            if self.tangent_space.should_defer_apply() {
                self.deferred_apply_state = Some(state);
                self.force_world_mode(camera);
            } else {
                self.apply_state(state, next_config, diff);
            }
        }

        self.tangent_space.update(
            input,
            dt,
            camera,
            display_manager,
            &self.grid_world,
            self.grid.get_coords(),
            self.renderer.projection,
        );
        //self.renderer.set_zoom_mix(self.tangent_space.scene_mix());
        self.rebuild_render_field();
        self.update_sphere();
    }

    pub fn render(&self, camera: &Camera) {
        self.renderer.render(
            &self.grid,
            &self.render_field,
            &self.render_form_samples,
            camera,
            &self.sphere,
            &self.scene_transform(),
        )
    }

    fn force_world_mode(&mut self, camera: &mut Camera) {
        self.tangent_space.force_world_mode(camera);
    }

    fn scene_transform(&self) -> SceneSpaceTransform {
        self.tangent_space.scene_transform()
    }

    fn update_sphere(&mut self) {
        if let Some(position) = self.tangent_space.marker_position() {
            self.sphere = Some(Sphere::new(position, WHITE, SPHERE_SIZE));
        } else {
            self.sphere = None;
        }
    }

    fn recompute_cached_field_vectors(&mut self) {
        self.cached_field_components.clear();
        self.cached_dual_components.clear();
        self.cached_field_vectors.clear();
        self.cached_field_components
            .reserve(self.field_samples.len());
        self.cached_dual_components
            .reserve(self.field_samples.len());
        self.cached_field_vectors.reserve(self.field_samples.len());

        for sample in &self.field_samples {
            let point = Point {
                x: sample.abstract_pos.x,
                y: sample.abstract_pos.y,
                z: sample.abstract_pos.z,
            };
            let vec_res = self.field.at(point);
            let dual_res = self.field.dual_at(point);
            let vector_components = Vector3::new(vec_res.x, vec_res.y, vec_res.z);
            let dual_components = Vector3::new(dual_res.x, dual_res.y, dual_res.z);
            self.cached_field_components.push(vector_components);
            self.cached_dual_components.push(dual_components);
            self.cached_field_vectors
                .push(sample.vector_to_world(vector_components));
        }
    }

    fn rebuild_render_field(&mut self) {
        let dual_scale = self.max_dual_abs_component();
        let show_form_samples = self.tangent_space.show_form_samples();

        self.render_field.clear();
        self.render_form_samples.clear();
        self.render_field.reserve(self.field_samples.len());
        self.render_form_samples.reserve(self.field_samples.len());

        for (((sample, field_components), dual_components), world_vector) in self
            .field_samples
            .iter()
            .zip(self.cached_field_components.iter().copied())
            .zip(self.cached_dual_components.iter().copied())
            .zip(self.cached_field_vectors.iter().copied())
        {
            let render_position = self
                .tangent_space
                .blend_position(sample.world_pos, sample.abstract_pos);
            let mut render_vector = self
                .tangent_space
                .blend_vector(world_vector, field_components);

            if self.normalize_field {
                let magnitude = render_vector.norm();
                if magnitude > 1e-6 {
                    render_vector /= magnitude;
                }
            }

            self.render_field.push(RenderVField::new(
                render_position,
                render_vector,
                Vector4::new(1.0, 1.0, 0.0, 1.0),
            ));

            if show_form_samples {
                let normalized_dual = dual_components / dual_scale;
                let size = FORM_SAMPLE_BASE_SIZE
                    + FORM_SAMPLE_SIZE_SCALE * (dual_components.norm() / dual_scale).sqrt();
                let color = differential_form_color(normalized_dual);
                self.render_form_samples
                    .push(Sphere::from_rgba(render_position, color, size));
            }
        }
    }

    pub fn get_projection(&self) -> Matrix4<f64> {
        self.renderer.projection
    }

    fn max_dual_abs_component(&self) -> f64 {
        self.cached_dual_components
            .iter()
            .flat_map(|value| [value.x.abs(), value.y.abs(), value.z.abs()])
            .fold(0.0, f64::max)
            .max(1.0e-6)
    }
}

fn differential_form_color(normalized_dual: Vector3<f64>) -> Vector4<f64> {
    let abs = normalized_dual.map(|value| value.abs().clamp(0.0, 1.0));
    let sign_balance =
        ((normalized_dual.x + normalized_dual.y + normalized_dual.z) / 3.0).clamp(-1.0, 1.0);

    let warm = vector![1.0, 0.72, 0.22];
    let cool = vector![0.15, 0.72, 1.0];
    let axis_color = vector![
        0.15 + 0.85 * abs.x,
        0.15 + 0.85 * abs.y,
        0.15 + 0.85 * abs.z
    ];

    let tint = if sign_balance >= 0.0 { warm } else { cool };
    let tint_strength = sign_balance.abs() * 0.35;
    let color = axis_color * (1.0 - tint_strength) + tint * tint_strength;
    Vector4::new(color.x, color.y, color.z, 0.95)
}

#[cfg(test)]
mod tests {
    use super::{differential_form_color, AppliedConfig};
    use crate::app::ui::GridUiState;
    use nalgebra::vector;

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

    #[test]
    fn differential_form_color_uses_axis_energy() {
        let color = differential_form_color(vector![1.0, 0.0, 0.0]);

        assert!(color.x > color.y);
        assert!(color.x > color.z);
    }
}
