//! Tangent-space transitions, hover picking, and geometric or dual tangent views.

mod dive;
mod types;

#[cfg(test)]
mod tests;

use crate::app::coords_sys::CoordsSys;
use crate::app::grid_world::{GridSample, GridWorld};
use crate::app::ui::legend::sampled_value_color;
use crate::app::ui::{LegendKind, LegendState};
use crate::graphics::model::Sphere;
use crate::toolbox::camera::Camera;
use crate::toolbox::input::Input;
use crate::toolbox::opengl::display_manager::DisplayManager;
use dive::{DiveAnchor, DiveMode, DiveState};
use glfw::Key;
use nalgebra::{vector, Matrix4, Vector3};
pub use types::{DualFormRender, SceneSpaceTransform, TangentRenderState, TangentView};

const DIVE_DURATION_SEC: f64 = 0.45;
const PICK_RADIUS: f64 = 0.45;
const PICK_LENGTH: f64 = 200.0;
const ZOOM_FACTOR: f64 = 0.8;
const MIN_ZOOM: f64 = 1.2;
const MAX_ZOOM: f64 = 8.0;
const MAX_ZOOM_FRACTION: f64 = 0.8;
const FORM_SAMPLE_SIZE: f64 = 0.06;
const DUAL_FORM_GRID_RADIUS: i32 = 4;
const DUAL_FORM_GRID_STEP: f64 = 0.45;
const DUAL_LOCAL_RADIUS: f64 = DUAL_FORM_GRID_RADIUS as f64 * DUAL_FORM_GRID_STEP;
const GEOMETRIC_LOCAL_RADIUS: f64 = 4.0;
const DEFAULT_GEOMETRIC_LOCAL_SCALE: f64 = 0.12;
const DEFAULT_GEOMETRIC_ARROW_SCALE: f64 = 0.55;

pub struct TangentSpace {
    hovered_sample: Option<GridSample>,
    dive: DiveState,
    geometric_local_scale: f64,
    geometric_arrow_scale: f64,
}

impl TangentSpace {
    /// Creates a new tangent-space controller.
    ///
    /// The controller starts in world mode and uses the default geometric patch and arrow scales
    /// until live UI settings overwrite them during `World::update`.
    pub fn new() -> Self {
        Self {
            hovered_sample: None,
            dive: DiveState::new(),
            geometric_local_scale: DEFAULT_GEOMETRIC_LOCAL_SCALE,
            geometric_arrow_scale: DEFAULT_GEOMETRIC_ARROW_SCALE,
        }
    }

    /// Updates the scale used to shrink geometric tangent-space positions around the anchor.
    ///
    /// The value is clamped away from zero to keep the local patch numerically stable.
    pub fn set_geometric_local_scale(&mut self, scale: f64) {
        self.geometric_local_scale = scale.max(1.0e-3);
    }

    /// Updates the multiplier applied to geometric tangent-space field arrows.
    ///
    /// The value is clamped away from zero so tangent vectors remain renderable.
    pub fn set_geometric_arrow_scale(&mut self, scale: f64) {
        self.geometric_arrow_scale = scale.max(1.0e-3);
    }

    /// Returns whether configuration changes should be deferred until tangent mode ends.
    ///
    /// Deferred application protects the invariants of an active dive: anchor position, tangent
    /// basis, cached camera endpoints, and picked sample all refer to the currently loaded grid.
    /// Rebuilding the grid underneath those values would make the transition inconsistent.
    pub fn should_defer_apply(&self) -> bool {
        self.dive.mode != DiveMode::World
    }

    /// Updates hover picking, tangent-mode requests, and dive animation state for the current
    /// frame.
    ///
    /// This code is deliberately self-contained and lock-free: it consumes the already-owned
    /// camera, display snapshot, grid lookup, and coordinate system, then updates only local
    /// tangent state. In world mode it performs hover picking; in tangent mode it preserves the
    /// dive camera relationship while still allowing user translation.
    pub fn update(
        &mut self,
        input: &Input,
        dt: f64,
        camera: &mut Camera,
        display_manager: &DisplayManager,
        grid_world: &GridWorld,
        coords: &CoordsSys,
        projection: Matrix4<f64>,
    ) {
        let requested_view = requested_view(input);
        match self.dive.mode {
            DiveMode::World => {
                camera.update(input);
                self.hovered_sample =
                    self.pick_hover_sample(camera, display_manager, grid_world, projection);
                if let Some(view) = requested_view {
                    if let Some(sample) = self.hovered_sample.clone() {
                        self.start_enter(camera, coords, sample, view);
                    }
                }
            }
            DiveMode::Tangent => {
                self.hovered_sample = None;
                let previous_position = camera.position;
                camera.update(input);
                let translation_delta = camera.position - previous_position;
                if let Some(endpoints) = self.dive.camera_endpoints.as_mut() {
                    endpoints.shift(translation_delta);
                }
                if let Some(view) = requested_view {
                    if view == self.dive.view {
                        self.dive.mode = DiveMode::Exiting;
                    } else {
                        self.dive.view = view;
                    }
                }
            }
            DiveMode::Entering | DiveMode::Exiting => {
                self.hovered_sample = None;
                if let Some(view) = requested_view {
                    match self.dive.mode {
                        DiveMode::Entering => {
                            if view == self.dive.view {
                                self.dive.reverse();
                            } else {
                                self.dive.view = view;
                            }
                        }
                        DiveMode::Exiting => {
                            self.dive.view = view;
                            self.dive.reverse();
                        }
                        DiveMode::World | DiveMode::Tangent => {}
                    }
                }
            }
        }

        self.dive.advance(dt.max(0.0), &mut camera.position);
    }

    /// Cancels any tangent transition and restores the world camera position.
    ///
    /// Hover state and anchor state are cleared so the subsystem returns to its neutral world-
    /// mode state.
    pub fn force_world_mode(&mut self, camera: &mut Camera) {
        if let Some(endpoints) = self.dive.camera_endpoints {
            camera.position = endpoints.world_pos;
        }
        self.dive.clear();
        self.hovered_sample = None;
    }

    /// Returns the eased scene blend used for rendering and camera interpolation.
    ///
    /// The raw animation alpha is passed through `smoothstep` so the transition starts and ends
    /// gently.
    pub fn scene_mix(&self) -> f64 {
        self.dive.scene_mix()
    }

    /// Returns the currently active tangent view, if the subsystem is not in world mode.
    ///
    /// World mode reports `None` so callers can branch cleanly between blended and unblended
    /// rendering.
    pub fn active_view(&self) -> Option<TangentView> {
        if self.dive.mode == DiveMode::World {
            None
        } else {
            Some(self.dive.view)
        }
    }

    /// Builds a compact snapshot of the tangent rendering state.
    ///
    /// The snapshot is used by the world to decide when cached renderables need to be rebuilt.
    pub fn render_state(&self) -> TangentRenderState {
        TangentRenderState {
            scene_mix: self.scene_mix(),
            active_view: self.active_view(),
            anchor_abstract_pos: self.anchor_abstract_position(),
            geometric_local_scale: self.geometric_local_scale,
            geometric_arrow_scale: self.geometric_arrow_scale,
        }
    }

    /// Returns the scene transform currently implied by the tangent subsystem.
    ///
    /// This compact value is what crosses into the grid renderer and shader. It contains only the
    /// blend amount, anchor, basis, and local scale needed to morph the grid on the GPU without
    /// duplicating scene state.
    pub fn scene_transform(&self) -> SceneSpaceTransform {
        if let Some(anchor) = &self.dive.anchor {
            anchor.scene_transform(self.dive.scene_mix(), self.geometric_local_scale)
        } else {
            SceneSpaceTransform::identity()
        }
    }

    /// Returns the point marker position that should be shown for the current tangent state.
    ///
    /// While diving this interpolates the anchor marker toward the origin; otherwise it shows
    /// the current hover sample.
    pub fn marker_position(&self) -> Option<Vector3<f64>> {
        if let Some(anchor) = &self.dive.anchor {
            Some(lerp_vec3(
                anchor.world_pos,
                Vector3::zeros(),
                self.dive.scene_mix(),
            ))
        } else {
            self.hovered_sample.as_ref().map(|sample| sample.world_pos)
        }
    }

    /// Returns the abstract-space position of the active tangent anchor, if any.
    ///
    /// Callers use this to sample fields and build tangent-only overlays around the anchor.
    pub fn anchor_abstract_position(&self) -> Option<Vector3<f64>> {
        self.dive.anchor.as_ref().map(|anchor| anchor.abstract_pos)
    }

    /// Blends a world-space position toward its tangent-space representation.
    ///
    /// Geometric view uses the anchor basis and local scale, while dual view uses an unscaled
    /// abstract delta.
    pub fn blend_position(
        &self,
        world_pos: Vector3<f64>,
        abstract_pos: Vector3<f64>,
    ) -> Vector3<f64> {
        if let Some(anchor) = &self.dive.anchor {
            let tangent_pos = if self.active_view() == Some(TangentView::Geometric) {
                anchor.geometric_tangent_position(abstract_pos, self.geometric_local_scale)
            } else {
                abstract_pos - anchor.abstract_pos
            };
            lerp_vec3(world_pos, tangent_pos, self.dive.scene_mix())
        } else {
            world_pos
        }
    }

    /// Blends a world-space vector toward its tangent-space representation.
    ///
    /// Geometric view rotates and scales tangent components in the anchor basis, while dual
    /// view keeps the raw components.
    pub fn blend_vector(
        &self,
        world_vector: Vector3<f64>,
        field_components: Vector3<f64>,
    ) -> Vector3<f64> {
        if let Some(anchor) = &self.dive.anchor {
            let tangent_vector = if self.active_view() == Some(TangentView::Geometric) {
                anchor.geometric_tangent_vector(field_components) * self.geometric_arrow_scale
            } else {
                field_components
            };
            lerp_vec3(world_vector, tangent_vector, self.dive.scene_mix())
        } else {
            world_vector
        }
    }

    /// Returns whether dual-form sample spheres should be rendered for the current blend state.
    ///
    /// Samples appear only in dual tangent mode once the transition is at least halfway
    /// complete.
    pub fn show_form_samples(&self) -> bool {
        self.active_view() == Some(TangentView::Dual) && self.scene_mix() >= 0.5
    }

    /// Returns whether vector arrows should be rendered for the current tangent state.
    ///
    /// This is the inverse of `show_form_samples`, because the dual view replaces arrows with
    /// sampled form values.
    pub fn show_vector_field(&self) -> bool {
        !self.show_form_samples()
    }

    /// Returns whether the grid should remain visible during the current tangent state.
    ///
    /// The grid fades out once dual tangent mode is sufficiently blended in.
    pub fn show_grid(&self) -> bool {
        self.active_view() != Some(TangentView::Dual) || self.scene_mix() < 0.5
    }

    /// Returns the number of lattice samples generated for dual-form rendering.
    ///
    /// The count is derived from the configured cubic sample radius and is used for buffer
    /// preallocation.
    pub fn dual_form_sample_capacity(&self) -> usize {
        ((2 * DUAL_FORM_GRID_RADIUS + 1).pow(3)) as usize
    }

    /// Blends field components toward their tangent-linearized approximation when needed.
    ///
    /// Only geometric tangent mode interpolates toward the local linearization; other modes
    /// keep the original components.
    pub fn blend_field_components(
        &self,
        field_components: Vector3<f64>,
        tangent_field_components: Option<Vector3<f64>>,
    ) -> Vector3<f64> {
        if self.active_view() == Some(TangentView::Geometric) {
            tangent_field_components
                .map(|tangent| lerp_vec3(field_components, tangent, self.scene_mix()))
                .unwrap_or(field_components)
        } else {
            field_components
        }
    }

    /// Returns whether one abstract-space field sample lies inside the active tangent patch.
    ///
    /// Outside tangent mode every sample remains visible. In geometric view the locality check
    /// is performed in the scaled tangent coordinates so the visible patch stays consistent with
    /// the rendered tangent grid. In dual view it is performed in the raw anchor-relative
    /// abstract coordinates.
    pub fn contains_local_sample(&self, abstract_pos: Vector3<f64>) -> bool {
        let Some(anchor) = self.dive.anchor.as_ref() else {
            return true;
        };

        let local_position = if self.active_view() == Some(TangentView::Geometric) {
            anchor.local_abstract_delta(abstract_pos, self.geometric_local_scale)
        } else {
            abstract_pos - anchor.abstract_pos
        };
        let radius = if self.active_view() == Some(TangentView::Geometric) {
            GEOMETRIC_LOCAL_RADIUS
        } else {
            DUAL_LOCAL_RADIUS
        };

        local_position.x.abs() <= radius
            && local_position.y.abs() <= radius
            && local_position.z.abs() <= radius
    }

    /// Returns the scaled abstract-space offset from the active anchor.
    ///
    /// This is only available while tangent mode has an anchor selected.
    pub fn geometric_local_delta(&self, abstract_pos: Vector3<f64>) -> Option<Vector3<f64>> {
        let anchor = self.dive.anchor.as_ref()?;
        Some(anchor.local_abstract_delta(abstract_pos, self.geometric_local_scale))
    }

    /// Returns the raw abstract-space offset from the active anchor.
    ///
    /// The delta is unscaled so callers can reuse it for dual-view and linearization work.
    pub fn abstract_delta(&self, abstract_pos: Vector3<f64>) -> Option<Vector3<f64>> {
        let anchor = self.dive.anchor.as_ref()?;
        Some(abstract_pos - anchor.abstract_pos)
    }

    /// Builds the sampled spheres and legend metadata for dual tangent rendering.
    ///
    /// The supplied dual-form components are sampled on a fixed local lattice centered at the
    /// active anchor. The resulting min/max range is returned alongside the spheres so the UI can
    /// render a legend without recomputing any field values itself.
    pub fn build_dual_form_render(&self, dual_components: Vector3<f64>) -> Option<DualFormRender> {
        let anchor = self.dive.anchor.as_ref()?;

        let dual_norm = dual_components.norm();
        let mut sampled_values = Vec::with_capacity(self.dual_form_sample_capacity());
        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;

        for z in -DUAL_FORM_GRID_RADIUS..=DUAL_FORM_GRID_RADIUS {
            for y in -DUAL_FORM_GRID_RADIUS..=DUAL_FORM_GRID_RADIUS {
                for x in -DUAL_FORM_GRID_RADIUS..=DUAL_FORM_GRID_RADIUS {
                    let tangent_position = vector![
                        x as f64 * DUAL_FORM_GRID_STEP,
                        y as f64 * DUAL_FORM_GRID_STEP,
                        z as f64 * DUAL_FORM_GRID_STEP
                    ];
                    let value = dual_components.dot(&tangent_position);
                    let render_position = anchor.geometric_tangent_vector(tangent_position);
                    min_value = min_value.min(value);
                    max_value = max_value.max(value);
                    sampled_values.push((render_position, value));
                }
            }
        }

        if dual_norm <= 1.0e-6 {
            min_value = -1.0;
            max_value = 1.0;
        }

        let mut samples = Vec::with_capacity(sampled_values.len());
        for (position, value) in sampled_values {
            let color = sampled_value_color(value, min_value, max_value);
            samples.push(Sphere::from_rgba(position, color, FORM_SAMPLE_SIZE));
        }

        Some(DualFormRender {
            samples,
            legend: LegendState {
                kind: LegendKind::DualTangent,
                min_value,
                max_value,
            },
        })
    }

    /// Casts the current mouse ray into the sampled grid and returns the hovered sample.
    ///
    /// The display manager and projection matrix are used to convert the cursor position into a
    /// world-space ray first.
    fn pick_hover_sample(
        &self,
        camera: &Camera,
        display_manager: &DisplayManager,
        grid_world: &GridWorld,
        projection: Matrix4<f64>,
    ) -> Option<GridSample> {
        let mouse_info = camera.mouse_pos_to_world_pos(display_manager, projection);
        grid_world.ray_cast(&mouse_info.0, &mouse_info.1, PICK_RADIUS, PICK_LENGTH)
    }

    /// Initializes a new dive from world space into one tangent view.
    ///
    /// The picked sample becomes the stable anchor for the entire transition. Its abstract
    /// coordinates, embedded world position, tangent basis, and camera-relative zoom offset are
    /// captured up front so later animation frames do not need to repick or rederive them.
    fn start_enter(
        &mut self,
        camera: &Camera,
        coords: &CoordsSys,
        sample: GridSample,
        view: TangentView,
    ) {
        let anchor = DiveAnchor {
            abstract_pos: sample.abstract_pos,
            world_pos: sample.world_pos,
            basis: coords.eval_tangent_basis(sample.abstract_pos),
            zoom_offset: compute_zoom_offset(camera.position, sample.world_pos),
        };
        self.dive.alpha = 0.0;
        self.dive.mode = DiveMode::Entering;
        self.dive.view = view;
        self.dive.camera_endpoints = Some(anchor.build_camera_endpoints(camera.position));
        self.dive.anchor = Some(anchor);
        self.hovered_sample = None;
    }
}

fn requested_view(input: &Input) -> Option<TangentView> {
    if !input.is_key_just_pressed(Key::T) {
        return None;
    }

    if input.is_key_pressed(Key::LeftControl) || input.is_key_pressed(Key::RightControl) {
        Some(TangentView::Dual)
    } else {
        Some(TangentView::Geometric)
    }
}

fn compute_zoom_offset(camera_pos: Vector3<f64>, anchor_world: Vector3<f64>) -> Vector3<f64> {
    let to_anchor = anchor_world - camera_pos;
    let distance = to_anchor.norm();
    if distance <= 1e-6 {
        return Vector3::zeros();
    }

    let zoom_amount = (distance * ZOOM_FACTOR).clamp(MIN_ZOOM, MAX_ZOOM);
    let zoom_amount = zoom_amount.min(distance * MAX_ZOOM_FRACTION);
    to_anchor.normalize() * zoom_amount
}

/// Linearly interpolates between two vectors.
///
/// The interpolation parameter is used directly without additional clamping.
fn lerp_vec3(from: Vector3<f64>, to: Vector3<f64>, t: f64) -> Vector3<f64> {
    from + (to - from) * t
}

/// Applies a cubic smoothstep easing curve to a scalar value.
///
/// Input values are clamped to `[0, 1]` before the eased blend is computed.
fn smoothstep(t: f64) -> f64 {
    let clamped = t.clamp(0.0, 1.0);
    clamped * clamped * (3.0 - 2.0 * clamped)
}
