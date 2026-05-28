//! Runtime world state that bridges UI changes, cached field data, and rendering.

mod apply;
mod field_rendering;
mod frame;
mod grid_cache;

use crate::app::applied_config::AppliedConfig;
use crate::app::coords_sys::CoordsSys;
use crate::app::em_runtime::EmRuntime;
use crate::app::field_render::{EmRenderCache, FieldRenderCache, FieldSample};
use crate::app::field_runtime::RuntimeField;
use crate::app::grid::Grid;
use crate::app::grid_world::{GridSample, GridWorld};
use crate::app::tangent_space::TangentSpace;
use crate::app::ui::{GridUiState, LegendState};
use crate::graphics::model::{RenderVField, Sphere};
use crate::render::master_render::MasterRenderer;
use crate::toolbox::opengl::display_manager::DisplayManager;
use nalgebra::Matrix4;
use std::sync::{Arc, Mutex};

const SPHERE_SIZE: f64 = 0.1;

pub struct World {
    field: RuntimeField,
    em_runtime: Option<EmRuntime>,
    render_field: Vec<RenderVField>,
    render_form_samples: Vec<Sphere>,
    field_samples: Vec<FieldSample>,
    field_cache: FieldRenderCache,
    em_cache: Option<EmRenderCache>,
    em_time: f64,
    last_em_reset_counter: u64,
    normalize_field: bool,
    em_normalize_vectors: bool,
    renderer: MasterRenderer,
    grid: Grid,
    grid_world: GridWorld,
    shared_ui_state: Arc<Mutex<GridUiState>>,
    deferred_apply_state: Option<GridUiState>,
    last_counter: u64,
    sphere: Option<Sphere>,
    tangent_space: TangentSpace,
    applied_config: AppliedConfig,
    legend: Option<LegendState>,
}

impl World {
    /// Creates the world, renderer, cached grid samples, and initial field render data.
    ///
    /// Construction performs a single blocking lock on the shared UI state to clone the initial
    /// configuration. After that, the world owns its runtime copies of geometry, field state,
    /// caches, and renderers on the main thread, so the shared mutex is not held during any
    /// expensive setup or rendering work.
    pub fn new(shared_ui_state: Arc<Mutex<GridUiState>>, display_manager: &DisplayManager) -> Self {
        let initial_state = shared_ui_state.lock().unwrap().clone();
        let applied_config = AppliedConfig::from_ui(&initial_state);
        let (grid, field, field_samples, grid_samples) = Self::init(initial_state.clone());
        let em_runtime = initial_state.em.enabled.then(|| {
            EmRuntime::from_ui_with_config(&initial_state.em, &grid, applied_config.grid_config)
        });
        let mut world = Self {
            field,
            em_runtime,
            render_field: Vec::new(),
            render_form_samples: Vec::new(),
            field_samples,
            field_cache: FieldRenderCache::Scalar(Vec::new()),
            em_cache: None,
            em_time: 0.0,
            last_em_reset_counter: initial_state.em.reset_counter,
            normalize_field: initial_state.normalize_field,
            em_normalize_vectors: initial_state.em.normalize_vectors,
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
            legend: None,
        };
        world
            .tangent_space
            .set_geometric_local_scale(initial_state.tangent_scale);
        world
            .tangent_space
            .set_geometric_arrow_scale(initial_state.geometric_arrow_scale);
        world.recompute_cached_field_data();
        world.recompute_cached_em_data();
        world.rebuild_render_field();
        world
    }

    /// Builds the initial grid, vector field, and sample caches from UI state.
    ///
    /// This is the one-shot bootstrap path shared by world construction before incremental
    /// updates take over.
    fn init(initial_state: GridUiState) -> (Grid, RuntimeField, Vec<FieldSample>, Vec<GridSample>) {
        let config = initial_state.to_grid_config();
        let x_eq = initial_state.coords_sys.x.eq.clone();
        let y_eq = initial_state.coords_sys.y.eq.clone();
        let z_eq = initial_state.coords_sys.z.eq.clone();
        let sys_coord = CoordsSys::new(x_eq, y_eq, z_eq);
        let mut grid = Grid::new(sys_coord);
        grid.update_config(&config);
        let (field_samples, grid_samples) = Self::build_grid_cache(&grid);
        let field = RuntimeField::from_ui(&initial_state, &grid);
        (grid, field, field_samples, grid_samples)
    }

    /// Returns the projection matrix currently owned by the master renderer.
    ///
    /// Callers use this when they need to perform picking or other view-dependent calculations
    /// outside the renderer.
    pub fn get_projection(&self) -> Matrix4<f64> {
        self.renderer.projection
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::app::applied_config::AppliedConfig;
    use crate::app::ui::{EmGauge, EmMode, FieldKind, GridUiState};
    use crate::maths::differential::Form;
    use crate::maths::field::VectorField;
    use crate::maths::space::Space;
    use crate::maths::Point;
    use mathhook_core::Parser;
    use nalgebra::vector;

    #[test]
    fn apply_diff_is_scoped_to_field_changes() {
        let current = AppliedConfig::from_ui(&GridUiState::default());
        let mut next_state = GridUiState::default();
        next_state.field.x.eq_str = "2".to_string();
        let next = AppliedConfig::from_ui(&next_state);

        let diff = current.diff(&next);

        assert!(diff.vector_changed);
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
        assert!(!diff.vector_changed);
        assert!(!diff.coords_changed);
        assert!(!diff.normalize_changed);
    }

    #[test]
    fn apply_diff_tracks_scalar_and_render_d_changes() {
        let current = AppliedConfig::from_ui(&GridUiState::default());
        let mut next_state = GridUiState::default();
        next_state.field_kind = FieldKind::Scalar;
        next_state.scalar_field.eq_str = "x * y".to_string();
        next_state.render_d = true;
        let next = AppliedConfig::from_ui(&next_state);

        let diff = current.diff(&next);

        assert!(diff.field_kind_changed);
        assert!(diff.scalar_changed);
        assert!(diff.render_d_changed);
    }

    #[test]
    fn apply_diff_tracks_em_enable_mode_and_equation_changes() {
        let current = AppliedConfig::from_ui(&GridUiState::default());
        let mut next_state = GridUiState::default();
        next_state.em.enabled = true;
        next_state.em.mode = EmMode::Electric;
        next_state.em.electric_field.x.eq_str = "sin(t)".to_string();
        let next = AppliedConfig::from_ui(&next_state);

        let diff = current.diff(&next);

        assert!(diff.em_enabled_changed);
        assert!(diff.em_mode_changed);
        assert!(diff.em_equations_changed);
    }

    #[test]
    fn apply_diff_does_not_consume_hidden_field_drafts_while_em_is_enabled() {
        let mut current_state = GridUiState::default();
        current_state.em.enabled = true;
        let current = AppliedConfig::from_ui(&current_state);

        let mut draft_state = current_state.clone();
        draft_state.field.x.eq_str = "2".to_string();
        let draft = AppliedConfig::from_ui(&draft_state);

        let diff = current.diff(&draft);

        assert!(!diff.vector_changed);

        draft_state.em.enabled = false;
        draft_state.field.x.eq = Parser::default().parse("2").unwrap();
        let disabled = AppliedConfig::from_ui(&draft_state);
        let disable_diff = current.diff(&disabled);

        assert!(disable_diff.em_enabled_changed);
        assert!(disable_diff.vector_changed);
        assert!(disable_diff.runtime_field_changed());
    }

    #[test]
    fn apply_diff_tracks_em_vector_normalization() {
        let current = AppliedConfig::from_ui(&GridUiState::default());
        let mut next_state = GridUiState::default();
        next_state.em.normalize_vectors = true;
        let next = AppliedConfig::from_ui(&next_state);

        let diff = current.diff(&next);

        assert!(diff.em_normalize_changed);
        assert!(!diff.em_runtime_changed());
        assert!(diff.em_render_changed());
    }

    #[test]
    fn apply_diff_treats_em_magnetic_scale_as_render_only() {
        let current = AppliedConfig::from_ui(&GridUiState::default());
        let mut next_state = GridUiState::default();
        next_state.em.magnetic_vector_scale = 2.0;
        let next = AppliedConfig::from_ui(&next_state);

        let diff = current.diff(&next);

        assert!(diff.em_magnetic_scale_changed);
        assert!(!diff.em_equations_changed);
        assert!(!diff.em_runtime_changed());
        assert!(diff.em_render_changed());
    }

    #[test]
    fn apply_diff_tracks_em_gauge_selection() {
        let current = AppliedConfig::from_ui(&GridUiState::default());
        let mut next_state = GridUiState::default();
        next_state.em.gauge = EmGauge::Lorenz;
        let next = AppliedConfig::from_ui(&next_state);

        let diff = current.diff(&next);

        assert!(diff.em_equations_changed);
        assert!(diff.em_runtime_changed());
    }

    #[test]
    fn field_dual_components_use_dual_basis_in_transformed_space() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let space = Space::new(parse("x + 2y"), parse("3y + z"), parse("4z"));
        let field = VectorField::from_otn(
            Form::new_otn(vec![parse("1"), parse("0"), parse("0")], 1),
            &space,
        );
        let point = Point {
            x: 0.5,
            y: -1.0,
            z: 2.0,
        };

        let primal = World::field_components_at(&field, point);
        let dual = World::field_dual_components_at(&field, point);

        assert_eq!(primal, vector![1.0, 0.0, 0.0]);
        assert_ne!(dual, primal);
    }
}
