use crate::app::ui::LegendState;
use crate::graphics::model::Sphere;
use nalgebra::Vector3;

use super::{DEFAULT_GEOMETRIC_LOCAL_SCALE, GEOMETRIC_LOCAL_RADIUS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TangentView {
    Geometric,
    Dual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TangentRenderState {
    pub scene_mix: f64,
    pub active_view: Option<TangentView>,
    pub anchor_abstract_pos: Option<Vector3<f64>>,
    pub geometric_local_scale: f64,
    pub geometric_arrow_scale: f64,
}

pub struct DualFormRender {
    pub samples: Vec<Sphere>,
    pub legend: LegendState,
}

#[derive(Clone, Copy)]
pub struct SceneSpaceTransform {
    pub tangent_mix: f64,
    pub tangent_anchor_abstract: Vector3<f64>,
    pub tangent_basis: [Vector3<f64>; 3],
    pub tangent_position_scale: f64,
    pub tangent_local_radius: f64,
}

impl SceneSpaceTransform {
    /// Builds the neutral scene-space transform used outside tangent mode.
    ///
    /// The returned value represents no blend, a zero anchor, the canonical basis, and the
    /// default geometric local scale.
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
            tangent_local_radius: f64::INFINITY,
        }
    }

    pub(super) fn for_anchor(
        tangent_mix: f64,
        tangent_anchor_abstract: Vector3<f64>,
        tangent_basis: [Vector3<f64>; 3],
        tangent_position_scale: f64,
    ) -> Self {
        Self {
            tangent_mix,
            tangent_anchor_abstract,
            tangent_basis,
            tangent_position_scale,
            tangent_local_radius: GEOMETRIC_LOCAL_RADIUS,
        }
    }
}
