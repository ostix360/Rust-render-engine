use crate::app::coords_sys::CoordsSys;
use crate::app::grid_world::{GridSample, GridWorld};
use crate::toolbox::camera::Camera;
use crate::toolbox::input::Input;
use crate::toolbox::opengl::display_manager::DisplayManager;
use glfw::Key;
use nalgebra::{vector, Matrix4, Vector3};

const DIVE_DURATION_SEC: f64 = 0.45;
const PICK_RADIUS: f64 = 0.45;
const PICK_LENGTH: f64 = 200.0;
const ZOOM_FACTOR: f64 = 0.8;
const MIN_ZOOM: f64 = 1.2;
const MAX_ZOOM: f64 = 8.0;
const MAX_ZOOM_FRACTION: f64 = 0.8;

#[derive(Clone, Copy)]
pub struct SceneSpaceTransform {
    pub tangent_mix: f64,
    pub tangent_anchor_abstract: Vector3<f64>,
    pub tangent_basis: [Vector3<f64>; 3],
}

impl SceneSpaceTransform {
    pub fn identity() -> Self {
        Self {
            tangent_mix: 0.0,
            tangent_anchor_abstract: Vector3::zeros(),
            tangent_basis: [
                vector![1.0, 0.0, 0.0],
                vector![0.0, 1.0, 0.0],
                vector![0.0, 0.0, 1.0],
            ],
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
    fn tangent_position(&self, abstract_pos: Vector3<f64>) -> Vector3<f64> {
        let delta = abstract_pos - self.abstract_pos;
        self.basis[0] * delta.x + self.basis[1] * delta.y + self.basis[2] * delta.z
    }

    fn tangent_vector(&self, vector: Vector3<f64>) -> Vector3<f64> {
        self.basis[0] * vector.x + self.basis[1] * vector.y + self.basis[2] * vector.z
    }

    fn build_camera_endpoints(&self, camera_pos: Vector3<f64>) -> DiveCameraEndpoints {
        DiveCameraEndpoints {
            world_pos: camera_pos,
            tangent_pos: camera_pos - self.world_pos + self.zoom_offset,
        }
    }

    fn scene_transform(&self, mix: f64) -> SceneSpaceTransform {
        SceneSpaceTransform {
            tangent_mix: mix,
            tangent_anchor_abstract: self.abstract_pos,
            tangent_basis: self.basis,
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
    anchor: Option<DiveAnchor>,
    camera_endpoints: Option<DiveCameraEndpoints>,
}

impl DiveState {
    fn new() -> Self {
        Self {
            mode: DiveMode::World,
            alpha: 0.0,
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
}

impl TangentSpace {
    pub fn new() -> Self {
        Self {
            hovered_sample: None,
            dive: DiveState::new(),
        }
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
        let toggle_pressed = input.is_key_just_pressed(Key::T);
        match self.dive.mode {
            DiveMode::World => {
                camera.update(input);
                self.hovered_sample =
                    self.pick_hover_sample(camera, display_manager, grid_world, projection);
                if toggle_pressed {
                    if let Some(sample) = self.hovered_sample.clone() {
                        self.start_enter(camera, coords, sample);
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
                if toggle_pressed {
                    self.dive.mode = DiveMode::Exiting;
                }
            }
            DiveMode::Entering | DiveMode::Exiting => {
                self.hovered_sample = None;
                if toggle_pressed {
                    self.dive.reverse();
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

    pub fn scene_transform(&self) -> SceneSpaceTransform {
        if let Some(anchor) = &self.dive.anchor {
            anchor.scene_transform(self.dive.scene_mix())
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

    pub fn blend_position(
        &self,
        world_pos: Vector3<f64>,
        abstract_pos: Vector3<f64>,
    ) -> Vector3<f64> {
        if let Some(anchor) = &self.dive.anchor {
            lerp_vec3(
                world_pos,
                anchor.tangent_position(abstract_pos),
                self.dive.scene_mix(),
            )
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
            lerp_vec3(
                world_vector,
                anchor.tangent_vector(field_components),
                self.dive.scene_mix(),
            )
        } else {
            world_vector
        }
    }

    pub fn show_form_samples(&self) -> bool {
        self.scene_mix() >= 0.5
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

    fn start_enter(&mut self, camera: &Camera, coords: &CoordsSys, sample: GridSample) {
        let anchor = DiveAnchor {
            abstract_pos: sample.abstract_pos,
            world_pos: sample.world_pos,
            basis: coords.eval_tangent_basis(sample.abstract_pos),
            zoom_offset: compute_zoom_offset(camera.position, sample.world_pos),
        };
        self.dive.alpha = 0.0;
        self.dive.mode = DiveMode::Entering;
        self.dive.camera_endpoints = Some(anchor.build_camera_endpoints(camera.position));
        self.dive.anchor = Some(anchor);
        self.hovered_sample = None;
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

#[cfg(test)]
mod tests {
    use super::{compute_zoom_offset, smoothstep, DiveAnchor, DiveMode, SceneSpaceTransform};
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
            anchor.tangent_position(anchor.abstract_pos),
            vector![0.0, 0.0, 0.0]
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
    }
}
