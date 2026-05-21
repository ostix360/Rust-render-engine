use super::{lerp_vec3, smoothstep, SceneSpaceTransform, TangentView, DIVE_DURATION_SEC};
use nalgebra::Vector3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DiveMode {
    World,
    Entering,
    Tangent,
    Exiting,
}

impl DiveMode {
    /// Returns whether the dive mode represents an active transition.
    ///
    /// Only entering and exiting states interpolate camera and scene values over time.
    pub(super) fn is_animating(self) -> bool {
        matches!(self, Self::Entering | Self::Exiting)
    }
}

#[derive(Clone)]
pub(super) struct DiveAnchor {
    pub(super) abstract_pos: Vector3<f64>,
    pub(super) world_pos: Vector3<f64>,
    pub(super) basis: [Vector3<f64>; 3],
    pub(super) zoom_offset: Vector3<f64>,
}

impl DiveAnchor {
    /// Computes the anchor-relative abstract offset scaled for the local tangent patch.
    ///
    /// Geometric tangent rendering uses this scaled delta to shrink the visible neighborhood
    /// around the anchor.
    pub(super) fn local_abstract_delta(
        &self,
        abstract_pos: Vector3<f64>,
        local_scale: f64,
    ) -> Vector3<f64> {
        (abstract_pos - self.abstract_pos) * local_scale
    }

    /// Projects an abstract position into the anchor-relative geometric tangent view.
    ///
    /// The point is first converted into a local delta and then expanded in the anchor basis.
    pub(super) fn geometric_tangent_position(
        &self,
        abstract_pos: Vector3<f64>,
        local_scale: f64,
    ) -> Vector3<f64> {
        let delta = self.local_abstract_delta(abstract_pos, local_scale);
        self.basis[0] * delta.x + self.basis[1] * delta.y + self.basis[2] * delta.z
    }

    /// Expands tangent-basis components into a world-oriented tangent vector.
    ///
    /// This keeps the tangent view aligned with the basis sampled at the dive anchor.
    pub(super) fn geometric_tangent_vector(&self, vector: Vector3<f64>) -> Vector3<f64> {
        self.basis[0] * vector.x + self.basis[1] * vector.y + self.basis[2] * vector.z
    }

    /// Builds the world and tangent camera positions for a dive transition.
    ///
    /// The tangent endpoint keeps the current camera offset relative to the anchor while
    /// applying the configured zoom offset.
    pub(super) fn build_camera_endpoints(&self, camera_pos: Vector3<f64>) -> DiveCameraEndpoints {
        DiveCameraEndpoints {
            world_pos: camera_pos,
            tangent_pos: camera_pos - self.world_pos + self.zoom_offset,
        }
    }

    /// Returns the scene transform currently implied by the tangent subsystem.
    ///
    /// Outside tangent mode this falls back to the identity transform used by the grid shader.
    pub(super) fn scene_transform(&self, mix: f64, local_scale: f64) -> SceneSpaceTransform {
        SceneSpaceTransform::for_anchor(mix, self.abstract_pos, self.basis, local_scale)
    }
}

#[derive(Clone, Copy)]
pub(super) struct DiveCameraEndpoints {
    pub(super) world_pos: Vector3<f64>,
    pub(super) tangent_pos: Vector3<f64>,
}

impl DiveCameraEndpoints {
    /// Interpolates the camera position between the stored world and tangent endpoints.
    ///
    /// The interpolation parameter is expected to already be eased by the caller when needed.
    fn position_at(&self, mix: f64) -> Vector3<f64> {
        lerp_vec3(self.world_pos, self.tangent_pos, mix)
    }

    /// Translates both stored camera endpoints by the same world-space delta.
    ///
    /// This keeps the dive animation coherent while the user moves inside tangent mode.
    pub(super) fn shift(&mut self, delta: Vector3<f64>) {
        self.world_pos += delta;
        self.tangent_pos += delta;
    }
}

pub(super) struct DiveState {
    pub(super) mode: DiveMode,
    pub(super) alpha: f64,
    pub(super) view: TangentView,
    pub(super) anchor: Option<DiveAnchor>,
    pub(super) camera_endpoints: Option<DiveCameraEndpoints>,
}

impl DiveState {
    /// Creates a new `DiveState` in ordinary world mode.
    ///
    /// No anchor or camera endpoints are captured until the user requests a tangent dive over a
    /// picked grid sample.
    pub(super) fn new() -> Self {
        Self {
            mode: DiveMode::World,
            alpha: 0.0,
            view: TangentView::Geometric,
            anchor: None,
            camera_endpoints: None,
        }
    }

    /// Returns the eased scene blend used for rendering and camera interpolation.
    ///
    /// The raw animation alpha is passed through `smoothstep` so the transition starts and ends
    /// gently.
    pub(super) fn scene_mix(&self) -> f64 {
        smoothstep(self.alpha)
    }

    /// Flips an in-progress dive transition to the opposite direction.
    ///
    /// Only entering and exiting states are affected; steady states are left unchanged.
    pub(super) fn reverse(&mut self) {
        self.mode = match self.mode {
            DiveMode::Entering => DiveMode::Exiting,
            DiveMode::Exiting => DiveMode::Entering,
            other => other,
        };
    }

    /// Resets the dive state back to the regular world view.
    ///
    /// Cached anchors and camera endpoints are discarded so a later dive starts from fresh
    /// state.
    pub(super) fn clear(&mut self) {
        self.mode = DiveMode::World;
        self.alpha = 0.0;
        self.anchor = None;
        self.camera_endpoints = None;
    }

    /// Advances the dive animation and updates the camera position in place.
    ///
    /// The transition alpha is clamped to `[0, 1]`, and the final state snaps cleanly to either
    /// world or tangent mode.
    pub(super) fn advance(&mut self, dt: f64, camera_position: &mut Vector3<f64>) {
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
