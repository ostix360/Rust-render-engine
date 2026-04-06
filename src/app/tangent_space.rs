use crate::app::coords_sys::CoordsSys;
use crate::app::grid_world::{GridSample, GridWorld};
use crate::app::ui::DualLegendState;
use crate::graphics::model::Sphere;
use crate::toolbox::camera::Camera;
use crate::toolbox::input::Input;
use crate::toolbox::opengl::display_manager::DisplayManager;
use glfw::Key;
use nalgebra::{vector, Matrix4, Vector3, Vector4};

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
const DEFAULT_GEOMETRIC_LOCAL_SCALE: f64 = 0.12;
const DEFAULT_GEOMETRIC_ARROW_SCALE: f64 = 0.55;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TangentView {
    Geometric,
    Dual,
}

pub struct DualFormRender {
    pub samples: Vec<Sphere>,
    pub legend: DualLegendState,
}

#[derive(Clone, Copy)]
pub struct SceneSpaceTransform {
    pub tangent_mix: f64,
    pub tangent_anchor_abstract: Vector3<f64>,
    pub tangent_basis: [Vector3<f64>; 3],
    pub tangent_position_scale: f64,
}

impl SceneSpaceTransform {
    pub fn identity() -> Self {
        Self {
            tangent_mix: 0.0,
            tangent_anchor_abstract: Vector3::zeros(),
            tangent_basis: [
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ],
            tangent_position_scale: DEFAULT_GEOMETRIC_LOCAL_SCALE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiveMode {
    World,
    Entering,
    Tangent,
    Exiting,
}

impl DiveMode {
    fn is_animating(self) -> bool {
        matches!(self, Self::Entering | Self::Exiting)
    }
}

#[derive(Clone)]
struct DiveAnchor {
    abstract_pos: Vector3<f64>,
    world_pos: Vector3<f64>,
    basis: [Vector3<f64>; 3],
    zoom_offset: Vector3<f64>,
}

impl DiveAnchor {
    fn local_abstract_delta(&self, abstract_pos: Vector3<f64>, local_scale: f64) -> Vector3<f64> {
        (abstract_pos - self.abstract_pos) * local_scale
    }

    fn geometric_tangent_position(
        &self,
        abstract_pos: Vector3<f64>,
        local_scale: f64,
    ) -> Vector3<f64> {
        let delta = self.local_abstract_delta(abstract_pos, local_scale);
        self.basis[0] * delta.x + self.basis[1] * delta.y + self.basis[2] * delta.z
    }

    fn geometric_tangent_vector(&self, vector: Vector3<f64>) -> Vector3<f64> {
        self.basis[0] * vector.x + self.basis[1] * vector.y + self.basis[2] * vector.z
    }

    fn build_camera_endpoints(&self, camera_pos: Vector3<f64>) -> DiveCameraEndpoints {
        DiveCameraEndpoints {
            world_pos: camera_pos,
            tangent_pos: camera_pos - self.world_pos + self.zoom_offset,
        }
    }

    fn scene_transform(&self, mix: f64, local_scale: f64) -> SceneSpaceTransform {
        SceneSpaceTransform {
            tangent_mix: mix,
            tangent_anchor_abstract: self.abstract_pos,
            tangent_basis: self.basis,
            tangent_position_scale: local_scale,
        }
    }
}

#[derive(Clone, Copy)]
struct DiveCameraEndpoints {
    world_pos: Vector3<f64>,
    tangent_pos: Vector3<f64>,
}

impl DiveCameraEndpoints {
    fn position_at(&self, mix: f64) -> Vector3<f64> {
        lerp_vec3(self.world_pos, self.tangent_pos, mix)
    }

    fn shift(&mut self, delta: Vector3<f64>) {
        self.world_pos += delta;
        self.tangent_pos += delta;
    }
}

struct DiveState {
    mode: DiveMode,
    alpha: f64,
    view: TangentView,
    anchor: Option<DiveAnchor>,
    camera_endpoints: Option<DiveCameraEndpoints>,
}

impl DiveState {
    fn new() -> Self {
        Self {
            mode: DiveMode::World,
            alpha: 0.0,
            view: TangentView::Geometric,
            anchor: None,
            camera_endpoints: None,
        }
    }

    fn scene_mix(&self) -> f64 {
        smoothstep(self.alpha)
    }

    fn reverse(&mut self) {
        self.mode = match self.mode {
            DiveMode::Entering => DiveMode::Exiting,
            DiveMode::Exiting => DiveMode::Entering,
            other => other,
        };
    }

    fn clear(&mut self) {
        self.mode = DiveMode::World;
        self.alpha = 0.0;
        self.anchor = None;
        self.camera_endpoints = None;
    }

    fn advance(&mut self, dt: f64, camera_position: &mut Vector3<f64>) {
        if !self.mode.is_animating() {
            return;
        }

        let delta_alpha = if DIVE_DURATION_SEC <= f64::EPSILON {
            1.0
        } else {
            dt / DIVE_DURATION_SEC
        };

        match self.mode {
            DiveMode::Entering => {
                self.alpha = (self.alpha + delta_alpha).min(1.0);
            }
            DiveMode::Exiting => {
                self.alpha = (self.alpha - delta_alpha).max(0.0);
            }
            DiveMode::World | DiveMode::Tangent => {}
        }

        if let Some(endpoints) = self.camera_endpoints {
            *camera_position = endpoints.position_at(self.scene_mix());
        }

        if self.alpha >= 1.0 {
            if let Some(endpoints) = self.camera_endpoints {
                *camera_position = endpoints.tangent_pos;
            }
            self.mode = DiveMode::Tangent;
        } else if self.alpha <= 0.0 {
            if let Some(endpoints) = self.camera_endpoints {
                *camera_position = endpoints.world_pos;
            }
            self.clear();
        }
    }
}

pub struct TangentSpace {
    hovered_sample: Option<GridSample>,
    dive: DiveState,
    geometric_local_scale: f64,
    geometric_arrow_scale: f64,
}

impl TangentSpace {
    pub fn new() -> Self {
        Self {
            hovered_sample: None,
            dive: DiveState::new(),
            geometric_local_scale: DEFAULT_GEOMETRIC_LOCAL_SCALE,
            geometric_arrow_scale: DEFAULT_GEOMETRIC_ARROW_SCALE,
        }
    }

    pub fn set_geometric_local_scale(&mut self, scale: f64) {
        self.geometric_local_scale = scale.max(1.0e-3);
    }

    pub fn set_geometric_arrow_scale(&mut self, scale: f64) {
        self.geometric_arrow_scale = scale.max(1.0e-3);
    }

    pub fn should_defer_apply(&self) -> bool {
        self.dive.mode != DiveMode::World
    }

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

    pub fn force_world_mode(&mut self, camera: &mut Camera) {
        if let Some(endpoints) = self.dive.camera_endpoints {
            camera.position = endpoints.world_pos;
        }
        self.dive.clear();
        self.hovered_sample = None;
    }

    pub fn scene_mix(&self) -> f64 {
        self.dive.scene_mix()
    }

    pub fn active_view(&self) -> Option<TangentView> {
        if self.dive.mode == DiveMode::World {
            None
        } else {
            Some(self.dive.view)
        }
    }

    pub fn scene_transform(&self) -> SceneSpaceTransform {
        if let Some(anchor) = &self.dive.anchor {
            anchor.scene_transform(self.dive.scene_mix(), self.geometric_local_scale)
        } else {
            SceneSpaceTransform::identity()
        }
    }

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

    pub fn anchor_abstract_position(&self) -> Option<Vector3<f64>> {
        self.dive.anchor.as_ref().map(|anchor| anchor.abstract_pos)
    }

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

    pub fn show_form_samples(&self) -> bool {
        self.active_view() == Some(TangentView::Dual) && self.scene_mix() >= 0.5
    }

    pub fn show_vector_field(&self) -> bool {
        !self.show_form_samples()
    }

    pub fn show_grid(&self) -> bool {
        self.active_view() != Some(TangentView::Dual) || self.scene_mix() < 0.5
    }

    pub fn dual_form_sample_capacity(&self) -> usize {
        ((2 * DUAL_FORM_GRID_RADIUS + 1).pow(3)) as usize
    }

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

    pub fn geometric_local_delta(&self, abstract_pos: Vector3<f64>) -> Option<Vector3<f64>> {
        let anchor = self.dive.anchor.as_ref()?;
        Some(anchor.local_abstract_delta(abstract_pos, self.geometric_local_scale))
    }

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
            let color = differential_form_color(value, min_value, max_value);
            samples.push(Sphere::from_rgba(position, color, FORM_SAMPLE_SIZE));
        }

        Some(DualFormRender {
            samples,
            legend: DualLegendState {
                min_value,
                max_value,
            },
        })
    }

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

fn lerp_vec3(from: Vector3<f64>, to: Vector3<f64>, t: f64) -> Vector3<f64> {
    from + (to - from) * t
}

fn smoothstep(t: f64) -> f64 {
    let clamped = t.clamp(0.0, 1.0);
    clamped * clamped * (3.0 - 2.0 * clamped)
}

fn differential_form_color(value: f64, min_value: f64, max_value: f64) -> Vector4<f64> {
    let mix = if (max_value - min_value).abs() <= 1.0e-6 {
        if value > 1.0e-6 {
            1.0
        } else if value < -1.0e-6 {
            0.0
        } else {
            0.5
        }
    } else {
        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    };

    let cold = vector![0.08, 0.22, 1.0];
    let neutral = vector![0.95, 0.95, 1.0];
    let warm = vector![1.0, 0.18, 0.08];
    let color = if mix < 0.5 {
        let local_mix = mix * 2.0;
        cold * (1.0 - local_mix) + neutral * local_mix
    } else {
        let local_mix = (mix - 0.5) * 2.0;
        neutral * (1.0 - local_mix) + warm * local_mix
    };
    Vector4::new(color.x, color.y, color.z, 0.95)
}

#[cfg(test)]
mod tests {
    use super::{
        compute_zoom_offset, requested_view, smoothstep, DiveAnchor, DiveMode, SceneSpaceTransform,
        TangentSpace, TangentView, DEFAULT_GEOMETRIC_ARROW_SCALE, DEFAULT_GEOMETRIC_LOCAL_SCALE,
        DUAL_FORM_GRID_RADIUS, DUAL_FORM_GRID_STEP,
    };
    use crate::toolbox::input::Input;
    use glfw::{Action, Key};
    use nalgebra::vector;

    #[test]
    fn tangent_anchor_maps_to_origin() {
        let anchor = DiveAnchor {
            abstract_pos: vector![2.0, 3.0, 4.0],
            world_pos: vector![10.0, 20.0, 30.0],
            basis: [
                vector![1.0, 0.0, 0.0],
                vector![0.0, 1.0, 0.0],
                vector![0.0, 0.0, 1.0],
            ],
            zoom_offset: vector![0.0, 0.0, 0.0],
        };

        assert_eq!(
            anchor.geometric_tangent_position(anchor.abstract_pos, DEFAULT_GEOMETRIC_LOCAL_SCALE),
            vector![0.0, 0.0, 0.0]
        );
    }

    #[test]
    fn tangent_positions_are_scaled_to_a_local_patch() {
        let anchor = DiveAnchor {
            abstract_pos: vector![1.0, 2.0, 3.0],
            world_pos: vector![0.0, 0.0, 0.0],
            basis: [
                vector![1.0, 0.0, 0.0],
                vector![0.0, 1.0, 0.0],
                vector![0.0, 0.0, 1.0],
            ],
            zoom_offset: vector![0.0, 0.0, 0.0],
        };

        assert_eq!(
            anchor
                .geometric_tangent_position(vector![3.0, 2.0, 3.0], DEFAULT_GEOMETRIC_LOCAL_SCALE),
            vector![2.0 * DEFAULT_GEOMETRIC_LOCAL_SCALE, 0.0, 0.0]
        );
    }

    #[test]
    fn camera_endpoints_apply_anchor_translation_and_zoom() {
        let anchor = DiveAnchor {
            abstract_pos: vector![0.0, 0.0, 0.0],
            world_pos: vector![4.0, 5.0, 6.0],
            basis: [
                vector![1.0, 0.0, 0.0],
                vector![0.0, 1.0, 0.0],
                vector![0.0, 0.0, 1.0],
            ],
            zoom_offset: vector![0.5, 0.0, 0.0],
        };

        let endpoints = anchor.build_camera_endpoints(vector![8.0, 9.0, 10.0]);

        assert_eq!(endpoints.world_pos, vector![8.0, 9.0, 10.0]);
        assert_eq!(endpoints.tangent_pos, vector![4.5, 4.0, 4.0]);
    }

    #[test]
    fn zoom_offset_is_clamped() {
        let offset = compute_zoom_offset(vector![0.0, 0.0, 0.0], vector![10.0, 0.0, 0.0]);

        assert_eq!(offset, vector![8.0, 0.0, 0.0]);
    }

    #[test]
    fn zoom_offset_never_overshoots_anchor() {
        let offset = compute_zoom_offset(vector![0.0, 0.0, 0.0], vector![1.0, 0.0, 0.0]);

        assert_eq!(offset, vector![0.8, 0.0, 0.0]);
    }

    #[test]
    fn smoothstep_respects_endpoints() {
        assert_eq!(smoothstep(0.0), 0.0);
        assert_eq!(smoothstep(1.0), 1.0);
    }

    #[test]
    fn dive_mode_animation_flags_are_scoped() {
        assert!(!DiveMode::World.is_animating());
        assert!(DiveMode::Entering.is_animating());
        assert!(!DiveMode::Tangent.is_animating());
        assert!(DiveMode::Exiting.is_animating());
    }

    #[test]
    fn identity_scene_transform_has_zero_mix() {
        let transform = SceneSpaceTransform::identity();

        assert_eq!(transform.tangent_mix, 0.0);
        assert_eq!(
            transform.tangent_position_scale,
            DEFAULT_GEOMETRIC_LOCAL_SCALE
        );
    }

    #[test]
    fn tangent_space_defaults_to_expected_arrow_scale() {
        let tangent_space = TangentSpace::new();

        assert_eq!(
            tangent_space.geometric_arrow_scale,
            DEFAULT_GEOMETRIC_ARROW_SCALE
        );
    }

    #[test]
    fn requested_view_defaults_to_geometric_tangent() {
        let mut input = Input::new();
        input.begin_frame();
        input.key_handler(Action::Press, Key::T);

        assert_eq!(requested_view(&input), Some(TangentView::Geometric));
    }

    #[test]
    fn requested_view_uses_ctrl_t_for_dual_tangent() {
        let mut input = Input::new();
        input.begin_frame();
        input.key_handler(Action::Press, Key::LeftControl);
        input.key_handler(Action::Press, Key::T);

        assert_eq!(requested_view(&input), Some(TangentView::Dual));
    }

    #[test]
    fn dual_view_hides_grid_once_transition_completes() {
        let mut tangent_space = TangentSpace::new();
        tangent_space.dive.mode = DiveMode::Tangent;
        tangent_space.dive.alpha = 1.0;
        tangent_space.dive.view = TangentView::Dual;

        assert!(!tangent_space.show_grid());
        assert!(tangent_space.show_form_samples());
        assert!(!tangent_space.show_vector_field());
    }

    #[test]
    fn geometric_tangent_vector_uses_anchor_basis_orientation() {
        let anchor = DiveAnchor {
            abstract_pos: vector![0.0, 0.0, 0.0],
            world_pos: vector![0.0, 0.0, 0.0],
            basis: [
                vector![0.0, 0.0, 1.0],
                vector![1.0, 0.0, 0.0],
                vector![0.0, 1.0, 0.0],
            ],
            zoom_offset: vector![0.0, 0.0, 0.0],
        };

        assert_eq!(
            anchor.geometric_tangent_vector(vector![1.0, 0.0, 0.0]),
            vector![0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn dual_form_render_uses_anchor_basis_for_sample_positions() {
        let mut tangent_space = TangentSpace::new();
        tangent_space.dive.anchor = Some(DiveAnchor {
            abstract_pos: vector![0.0, 0.0, 0.0],
            world_pos: vector![0.0, 0.0, 0.0],
            basis: [
                vector![0.0, 0.0, 1.0],
                vector![1.0, 0.0, 0.0],
                vector![0.0, 1.0, 0.0],
            ],
            zoom_offset: vector![0.0, 0.0, 0.0],
        });

        let render = tangent_space
            .build_dual_form_render(vector![1.0, 0.0, 0.0])
            .expect("dual render");

        let expected = vector![0.0, 0.0, DUAL_FORM_GRID_STEP * DUAL_FORM_GRID_RADIUS as f64];
        assert!(render
            .samples
            .iter()
            .any(|sample| sample.position == expected));
    }
}
