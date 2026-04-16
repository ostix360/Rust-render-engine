//! Runtime world state that bridges UI changes, cached field data, and rendering.

use crate::app::coords_sys::CoordsSys;
use crate::app::field_runtime::RuntimeField;
use crate::app::grid::{Grid, GridConfig};
use crate::app::grid_world::{GridSample, GridWorld};
use crate::app::tangent_space::{SceneSpaceTransform, TangentRenderState, TangentSpace};
use crate::app::ui::legend::sampled_value_color;
use crate::app::ui::{FieldKind, GridUiState, LegendKind, LegendState};
use crate::graphics::model::{RenderVField, Sphere};
use crate::maths::field::VectorField;
use crate::maths::Point;
use crate::render::master_render::MasterRenderer;
use crate::toolbox::camera::Camera;
use crate::toolbox::color::WHITE;
use crate::toolbox::input::Input;
use crate::toolbox::opengl::display_manager::DisplayManager;
use mathhook_core::formatter::simple::SimpleContext;
use mathhook_core::SimpleFormatter;
use nalgebra::{Matrix4, Vector3, Vector4};
use std::sync::{Arc, Mutex};
use typed_floats::NonNaN;

const SPHERE_SIZE: f64 = 0.1;

#[derive(Clone, PartialEq)]
struct AppliedConfig {
    grid_config: GridConfig,
    coord_eqs: [String; 3],
    field_kind: FieldKind,
    scalar_eq: String,
    vector_eqs: [String; 3],
    render_d: bool,
    normalize_field: bool,
}

impl AppliedConfig {
    /// Builds the subset of UI state that drives world reconfiguration.
    ///
    /// The stored values are normalized into comparable strings and flags so cheap change
    /// detection can happen frame-to-frame.
    fn from_ui(state: &GridUiState) -> Self {
        let context = SimpleContext::default();
        Self {
            grid_config: state.to_grid_config(),
            coord_eqs: [
                state
                    .coords_sys
                    .x
                    .eq
                    .to_simple(&context)
                    .expect("Error while converting x eq"),
                state
                    .coords_sys
                    .y
                    .eq
                    .to_simple(&context)
                    .expect("Error while converting y eq"),
                state
                    .coords_sys
                    .z
                    .eq
                    .to_simple(&context)
                    .expect("Error while converting z eq"),
            ],
            field_kind: state.field_kind,
            scalar_eq: state.scalar_field.eq_str.clone(),
            vector_eqs: [
                state.field.x.eq_str.clone(),
                state.field.y.eq_str.clone(),
                state.field.z.eq_str.clone(),
            ],
            render_d: state.render_d,
            normalize_field: state.normalize_field,
        }
    }

    /// Computes which high-level parts of the world would change between two applied
    /// configurations.
    ///
    /// The result is used to decide whether geometry, field caches, or normalization settings
    /// need to be rebuilt.
    fn diff(&self, next: &Self) -> ApplyDiff {
        ApplyDiff {
            grid_changed: self.grid_config != next.grid_config,
            coords_changed: self.coord_eqs != next.coord_eqs,
            field_kind_changed: self.field_kind != next.field_kind,
            scalar_changed: self.scalar_eq != next.scalar_eq,
            vector_changed: self.vector_eqs != next.vector_eqs,
            render_d_changed: self.render_d != next.render_d,
            normalize_changed: self.normalize_field != next.normalize_field,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ApplyDiff {
    grid_changed: bool,
    coords_changed: bool,
    field_kind_changed: bool,
    scalar_changed: bool,
    vector_changed: bool,
    render_d_changed: bool,
    normalize_changed: bool,
}

impl ApplyDiff {
    /// Returns whether the grid geometry or coordinate embedding changed.
    ///
    /// Geometry changes force the world to rebuild sampled positions and the grid lookup cache.
    fn geometry_changed(self) -> bool {
        self.grid_changed || self.coords_changed
    }

    /// Returns whether the active runtime field must be rebuilt.
    fn runtime_field_changed(self) -> bool {
        self.coords_changed
            || self.field_kind_changed
            || self.scalar_changed
            || self.vector_changed
            || self.render_d_changed
    }

    /// Returns whether cached field samples must be recomputed.
    ///
    /// Any geometry change invalidates the cache in addition to direct field-expression
    /// changes.
    fn field_cache_changed(self) -> bool {
        self.geometry_changed() || self.runtime_field_changed()
    }
}

#[derive(Clone)]
struct FieldSample {
    abstract_pos: Vector3<f64>,
    world_pos: Vector3<f64>,
    basis: [Vector3<f64>; 3],
}

impl FieldSample {
    /// Expands basis components into a world-space vector at this sample.
    ///
    /// The sample stores the tangent basis that was evaluated when the cache was built.
    fn vector_to_world(&self, vector: Vector3<f64>) -> Vector3<f64> {
        self.basis[0] * vector.x + self.basis[1] * vector.y + self.basis[2] * vector.z
    }
}

/// Returns whether every component of the vector is finite.
fn is_finite_vec3(vector: &Vector3<f64>) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

pub struct World {
    field: RuntimeField,
    render_field: Vec<RenderVField>,
    render_form_samples: Vec<Sphere>,
    field_samples: Vec<FieldSample>,
    cached_scalar_values: Vec<f64>,
    cached_field_components: Vec<Vector3<f64>>,
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
        let mut world = Self {
            field,
            render_field: Vec::new(),
            render_form_samples: Vec::new(),
            field_samples,
            cached_scalar_values: Vec::new(),
            cached_field_components: Vec::new(),
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
            legend: None,
        };
        world
            .tangent_space
            .set_geometric_local_scale(initial_state.tangent_scale);
        world
            .tangent_space
            .set_geometric_arrow_scale(initial_state.geometric_arrow_scale);
        world.recompute_cached_field_data();
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

    /// Builds cached field and grid samples from the current grid render data.
    ///
    /// Each cached sample stores both abstract-space and world-space information so later
    /// updates can avoid recomputing geometry every frame.
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
                let world_pos = coords.eval_position(abstract_pos);
                let basis = coords.eval_tangent_basis(abstract_pos);

                if !is_finite_vec3(&world_pos) || basis.iter().any(|axis| !is_finite_vec3(axis)) {
                    continue;
                }

                field_samples.push(FieldSample {
                    abstract_pos,
                    world_pos,
                    basis,
                });

                for vertex in vertices.iter() {
                    let abstract_vertex = Self::transform_vertex(transform, vertex);
                    let world_vertex = coords.eval_position(abstract_vertex);
                    if !is_finite_vec3(&world_vertex) {
                        continue;
                    }
                    grid_samples.push(GridSample {
                        world_pos: world_vertex,
                        abstract_pos: abstract_vertex,
                    });
                }
            }
        }

        (field_samples, grid_samples)
    }

    /// Applies one segment transform to an abstract-space edge vertex.
    ///
    /// This converts the stored homogeneous transform and non-NaN vertex coordinates into a
    /// plain `Vector3<f64>`.
    fn transform_vertex(transform: &Matrix4<f64>, vertex: &Vector3<NonNaN<f64>>) -> Vector3<f64> {
        let vec4 = Vector4::new(vertex.x.get(), vertex.y.get(), vertex.z.get(), 1.0);
        (transform * vec4).xyz()
    }

    /// Applies validated UI state to the world and refreshes whichever caches changed.
    ///
    /// The caller is expected to pass in a fully cloned and validated `GridUiState`, so this
    /// method does not acquire the shared UI lock itself. That keeps the critical section short:
    /// lock on the UI thread, clone, unlock, then do the potentially expensive grid, kd-tree,
    /// shader, and field-cache rebuilds here on the render thread.
    ///
    /// Concretely:
    ///
    /// - coordinate edits rebuild `CoordsSys` and hot-reload the editable grid vertex shader,
    /// - geometry edits rebuild sampled grid data and the `GridWorld` picking kd-tree,
    /// - field edits rebuild the active runtime field,
    /// - field-cache changes recompute sampled vectors used by later render rebuilds.
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

        if diff.runtime_field_changed() {
            self.field = RuntimeField::from_ui(&state, &self.grid);
        }

        self.normalize_field = next_config.normalize_field;

        if diff.field_cache_changed() {
            self.recompute_cached_field_data();
        }

        self.applied_config = next_config;
        self.last_counter = state.apply_counter;
    }

    /// Advances the world by one frame.
    ///
    /// The shared UI mutex is only held long enough to copy live scalar settings and detect a new
    /// committed `apply_counter`. If a new configuration is pending while tangent mode is active,
    /// the state is stashed in `deferred_apply_state` and applied only after tangent mode is
    /// forced back to world space. This avoids rebuilding geometry or field caches while the dive
    /// transition still depends on the old anchor and camera endpoints.
    ///
    /// Outside of that lock boundary, this method owns the full frame update: applying pending
    /// state, advancing tangent-space logic, invalidating render buffers when needed, and pushing
    /// overlay metadata back to the UI.
    pub fn update(
        &mut self,
        input: &Input,
        dt: f64,
        display_manager: &DisplayManager,
        camera: &mut Camera,
    ) {
        let render_state_before = self.render_state();
        let mut needs_render_rebuild = false;
        let mut pending_state = self.deferred_apply_state.take();
        {
            let shared = self.shared_ui_state.lock().unwrap();
            self.tangent_space
                .set_geometric_local_scale(shared.tangent_scale);
            self.tangent_space
                .set_geometric_arrow_scale(shared.geometric_arrow_scale);
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
                needs_render_rebuild = true;
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
        //self.renderer.set_zoom_mix(self.tangent_space.scene_mix()); comment for now do not remove it!!
        if needs_render_rebuild || self.render_state() != render_state_before {
            self.rebuild_render_field();
        }
        self.sync_overlay_state();
        self.update_sphere();
    }

    /// Renders the current grid, field, tangent overlays, and marker sphere.
    ///
    /// Visibility of each layer is delegated to the tangent-space subsystem so world and
    /// tangent views stay synchronized.
    pub fn render(&self, camera: &Camera) {
        self.renderer.render(
            &self.grid,
            &self.render_field,
            &self.render_form_samples,
            self.tangent_space.show_grid(),
            self.show_vector_field(),
            camera,
            &self.sphere,
            &self.scene_transform(),
        )
    }

    /// Forces the tangent subsystem back to world mode.
    ///
    /// This is used when deferred UI changes must be applied but the current tangent transition
    /// still owns camera state.
    fn force_world_mode(&mut self, camera: &mut Camera) {
        self.tangent_space.force_world_mode(camera);
    }

    /// Returns the scene transform that should be supplied to the grid renderer.
    ///
    /// The transform mirrors the blend state maintained by the tangent subsystem.
    fn scene_transform(&self) -> SceneSpaceTransform {
        self.tangent_space.scene_transform()
    }

    /// Refreshes the marker sphere that highlights the hovered or anchored sample.
    ///
    /// The marker is derived from tangent-space state each frame instead of being shared through
    /// the UI mutex, so there is no cross-thread ownership of renderable scene objects.
    fn update_sphere(&mut self) {
        if let Some(position) = self.tangent_space.marker_position() {
            self.sphere = Some(Sphere::new(position, WHITE, SPHERE_SIZE));
        } else {
            self.sphere = None;
        }
    }

    /// Recomputes cached scalar values or vector components for every sampled point.
    ///
    /// This is one of the more expensive CPU-side rebuild steps: the field is evaluated in
    /// abstract coordinates, then expanded through the cached tangent basis stored in each
    /// `FieldSample`. The result is retained so tangent/view-only changes can rebuild renderables
    /// without reevaluating the field function itself.
    fn recompute_cached_field_data(&mut self) {
        self.cached_scalar_values.clear();
        self.cached_field_components.clear();
        self.cached_field_vectors.clear();
        match &self.field {
            RuntimeField::Scalar(field) => {
                self.cached_scalar_values.reserve(self.field_samples.len());
                for sample in &self.field_samples {
                    let value = field.at(Point {
                        x: sample.abstract_pos.x,
                        y: sample.abstract_pos.y,
                        z: sample.abstract_pos.z,
                    });
                    self.cached_scalar_values.push(value);
                }
            }
            RuntimeField::Vector(field) => {
                self.cached_field_components.reserve(self.field_samples.len());
                self.cached_field_vectors.reserve(self.field_samples.len());

                for sample in &self.field_samples {
                    let point = Point {
                        x: sample.abstract_pos.x,
                        y: sample.abstract_pos.y,
                        z: sample.abstract_pos.z,
                    };
                    let vec_res = field.at(point);
                    let vector_components = Vector3::new(vec_res.x, vec_res.y, vec_res.z);
                    let world_vector = sample.vector_to_world(vector_components);
                    self.cached_field_components.push(vector_components);
                    self.cached_field_vectors.push(world_vector);
                }
            }
        }
    }

    /// Rebuilds the current field renderables from the cached samples and tangent state.
    ///
    /// This stage is intentionally downstream from `recompute_cached_field_vectors`: cached field
    /// values are turned into render-oriented arrows or dual-form spheres after tangent blending,
    /// normalization, and anchor-relative transforms are known. That split keeps camera and
    /// tangent-view changes cheaper than full field recomputation.
    fn rebuild_render_field(&mut self) {
        self.render_field.clear();
        self.render_form_samples.clear();
        self.legend = None;
        self.render_field.reserve(self.field_samples.len());
        self.render_form_samples
            .reserve(self.tangent_space.dual_form_sample_capacity());
        match self.field.clone() {
            RuntimeField::Scalar(_) => self.rebuild_scalar_field_samples(),
            RuntimeField::Vector(field) => {
                self.rebuild_vector_render_field(&field);
                if self.tangent_space.show_form_samples() {
                    self.rebuild_dual_form_samples(self.anchor_dual_components());
                }
            }
        }
    }

    /// Returns whether arrows should be rendered for the active field mode.
    fn show_vector_field(&self) -> bool {
        self.field.is_vector_like() && self.tangent_space.show_vector_field()
    }

    /// Rebuilds colored scalar samples from the cached scalar field values.
    fn rebuild_scalar_field_samples(&mut self) {
        if self.cached_scalar_values.is_empty() {
            return;
        }

        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;
        for value in &self.cached_scalar_values {
            if !value.is_finite() {
                continue;
            }
            min_value = min_value.min(*value);
            max_value = max_value.max(*value);
        }

        if !min_value.is_finite() || !max_value.is_finite() {
            return;
        }

        for (sample, value) in self
            .field_samples
            .iter()
            .zip(self.cached_scalar_values.iter().copied())
        {
            if !value.is_finite() {
                continue;
            }
            let position = self
                .tangent_space
                .blend_position(sample.world_pos, sample.abstract_pos);
            if !is_finite_vec3(&position) {
                continue;
            }
            let color = sampled_value_color(value, min_value, max_value);
            self.render_form_samples
                .push(Sphere::from_rgba(position, color, SPHERE_SIZE * 0.55));
        }

        self.legend = Some(LegendState {
            kind: LegendKind::ScalarField,
            min_value,
            max_value,
        });
    }

    /// Rebuilds arrow renderables for the active vector-like field mode.
    fn rebuild_vector_render_field(&mut self, field: &VectorField) {
        let anchor_point = self.anchor_point();

        for ((sample, field_components), world_vector) in self
            .field_samples
            .iter()
            .zip(self.cached_field_components.iter().copied())
            .zip(self.cached_field_vectors.iter().copied())
        {
            let tangent_field_components = anchor_point.and_then(|anchor| {
                self.tangent_space
                    .abstract_delta(sample.abstract_pos)
                    .map(|delta| Self::field_linearized_components_at(field, anchor, delta))
            });
            let tangent_components = self
                .tangent_space
                .blend_field_components(field_components, tangent_field_components);
            let render_position = self
                .tangent_space
                .blend_position(sample.world_pos, sample.abstract_pos);
            let mut render_vector = self
                .tangent_space
                .blend_vector(world_vector, tangent_components);

            if !is_finite_vec3(&render_position) || !is_finite_vec3(&render_vector) {
                continue;
            }

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
        }
    }

    /// Returns the projection matrix currently owned by the master renderer.
    ///
    /// Callers use this when they need to perform picking or other view-dependent calculations
    /// outside the renderer.
    pub fn get_projection(&self) -> Matrix4<f64> {
        self.renderer.projection
    }

    /// Rebuilds the dual-form sample spheres and legend from anchor-space field data.
    ///
    /// If no anchor or no dual-form render can be produced, the previous sample buffer remains
    /// empty.
    fn rebuild_dual_form_samples(&mut self, anchor_field_components: Option<Vector3<f64>>) {
        let Some(dual_components) = anchor_field_components else {
            return;
        };
        let Some(render) = self.tangent_space.build_dual_form_render(dual_components) else {
            return;
        };

        self.legend = Some(render.legend);
        self.render_form_samples = render.samples;
    }

    /// Publishes overlay metadata back to the shared UI state.
    ///
    /// The shared lock is taken only for the scalar legend payload; renderables themselves remain
    /// owned by the main thread. This keeps the UI thread informed without turning the mutex into
    /// a transport for large scene structures.
    fn sync_overlay_state(&self) {
        let mut shared = self.shared_ui_state.lock().unwrap();
        shared.legend = self.legend;
    }

    /// Returns the tangent render-state snapshot used to detect render-cache invalidation.
    ///
    /// The snapshot is compared across frames to decide whether field geometry needs to be
    /// regenerated.
    fn render_state(&self) -> TangentRenderState {
        self.tangent_space.render_state()
    }

    /// Evaluates the field dual components at the current tangent anchor.
    ///
    /// This is only available while tangent mode has a selected anchor point.
    fn anchor_dual_components(&self) -> Option<Vector3<f64>> {
        let point = self.anchor_point()?;
        Some(Self::field_dual_components_at(self.field.as_vector()?, point))
    }

    /// Returns the current tangent anchor as a scalar `Point`.
    ///
    /// The conversion keeps the anchor in abstract coordinates so field evaluation stays
    /// consistent with the grid basis.
    fn anchor_point(&self) -> Option<Point> {
        let anchor_abstract = self.tangent_space.anchor_abstract_position()?;
        Some(Point {
            x: anchor_abstract.x,
            y: anchor_abstract.y,
            z: anchor_abstract.z,
        })
    }

    #[cfg(test)]
    fn field_components_at(field: &VectorField, point: Point) -> Vector3<f64> {
        let field_res = field.at(point);
        Vector3::new(field_res.x, field_res.y, field_res.z)
    }

    /// Evaluates the field in the dual basis at one abstract point.
    ///
    /// The returned vector is used to build dual tangent overlays and legends.
    fn field_dual_components_at(field: &VectorField, point: Point) -> Vector3<f64> {
        let field_res = field.dual_at(point);
        Vector3::new(field_res.x, field_res.y, field_res.z)
    }

    /// Evaluates the first-order field approximation around an anchor point.
    ///
    /// The supplied `local_delta` is interpreted in abstract coordinates relative to the
    /// anchor.
    fn field_linearized_components_at(
        field: &VectorField,
        anchor: Point,
        local_delta: Vector3<f64>,
    ) -> Vector3<f64> {
        let field_res = field.linearized_at(
            anchor,
            Point {
                x: local_delta.x,
                y: local_delta.y,
                z: local_delta.z,
            },
        );
        Vector3::new(field_res.x, field_res.y, field_res.z)
    }
}

#[cfg(test)]
mod tests {
    use super::AppliedConfig;
    use super::World;
    use crate::app::ui::{FieldKind, GridUiState};
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
    fn field_dual_components_use_dual_basis_in_transformed_space() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let space = Space::new(parse("x + 2y"), parse("3y + z"), parse("4z"));
        let field = VectorField::from_otn(
            Form::new(vec![parse("1"), parse("0"), parse("0")], 1),
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
