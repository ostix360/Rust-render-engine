use super::fields::{TimedScalarField, TimedVectorField};
use crate::app::ui::EmGauge;
use crate::maths::Point;
use nalgebra::Vector3;
use std::sync::Arc;

/// Builds a cheap local visualization potential from a magnetic field sample.
///
/// In electric source mode `B` is already a finite-domain inverse-curl field. Running the same
/// inverse-curl solver again for `A` would nest the quadrature and make the default visible
/// `A` layer quadratic in the Maxwell cell count. This local Coulomb-style potential keeps the
/// layer responsive while preserving a meaningful nonzero vector-potential visualization.
pub(super) fn local_vector_potential_from_b(magnetic_field: TimedVectorField) -> TimedVectorField {
    TimedVectorField::from_vector_expr(Arc::new(move |x, y, z, t| {
        let magnetic = magnetic_field.at(Point { x, y, z }, t);
        Vector3::new(
            0.5 * (magnetic.y * z - magnetic.z * y),
            0.5 * (magnetic.z * x - magnetic.x * z),
            0.5 * (magnetic.x * y - magnetic.y * x),
        )
    }))
}

fn local_scalar_potential_from_e(electric_field: TimedVectorField) -> TimedScalarField {
    TimedScalarField::from_fast_expr(Arc::new(move |x, y, z, t| {
        let electric = electric_field.at(Point { x, y, z }, t);
        -(electric.x * x + electric.y * y + electric.z * z)
    }))
}

pub(super) fn scalar_potential_for_gauge(
    gauge: EmGauge,
    electric_field: TimedVectorField,
) -> TimedScalarField {
    match gauge {
        EmGauge::Coulomb => local_scalar_potential_from_e(electric_field),
        // A zero Lorenz scalar potential would require a matching transform of `A`; otherwise
        // the displayed potentials no longer reconstruct the displayed source field.
        EmGauge::Lorenz => todo!("Lorenz gauge not yet implemented"),
    }
}
