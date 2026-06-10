use super::fields::{TimedScalarField, TimedVectorField};
use crate::app::coords_sys::CoordSampleGeometry;
use crate::app::ui::EmGauge;
use crate::maths::{Expr, Point};
use nalgebra::Vector3;
use std::sync::Arc;

const SCALAR_POTENTIAL_LINE_STEPS: usize = 24;
const VECTOR_POTENTIAL_TIME_EPSILON: f64 = 1.0e-4;

pub(super) fn zero_scalar_potential() -> TimedScalarField {
    TimedScalarField::new(Expr::number(0.0))
}

fn scalar_potential_from_reconstruction_residual(
    electric_field: TimedVectorField,
    vector_potential: TimedVectorField,
    geometry: CoordSampleGeometry,
) -> TimedScalarField {
    TimedScalarField::from_fast_expr(Arc::new(move |x, y, z, t| {
        let residual = |point: Point| {
            -electric_field.at(point, t)
                - vector_potential_time_derivative(&vector_potential, point, t)
        };
        line_integral_component(
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Vector3::new(1.0, 0.0, 0.0),
            0,
            x,
            &residual,
            &geometry,
        ) + line_integral_component(
            Point { x, y: 0.0, z: 0.0 },
            Vector3::new(0.0, 1.0, 0.0),
            1,
            y,
            &residual,
            &geometry,
        ) + line_integral_component(
            Point { x, y, z: 0.0 },
            Vector3::new(0.0, 0.0, 1.0),
            2,
            z,
            &residual,
            &geometry,
        )
    }))
}

pub(super) fn scalar_potential_for_gauge(
    gauge: EmGauge,
    electric_field: TimedVectorField,
    vector_potential: TimedVectorField,
    geometry: CoordSampleGeometry,
) -> TimedScalarField {
    match gauge {
        // A zero Lorenz scalar potential would require a matching transform of `A`; otherwise
        // the displayed potentials no longer reconstruct the displayed source field.
        EmGauge::Coulomb => scalar_potential_from_reconstruction_residual(
            electric_field,
            vector_potential,
            geometry,
        ),
    }
}

fn vector_potential_time_derivative(
    vector_potential: &TimedVectorField,
    point: Point,
    time: f64,
) -> Vector3<f64> {
    let forward = vector_potential.at(point, time + VECTOR_POTENTIAL_TIME_EPSILON);
    let backward = vector_potential.at(point, time - VECTOR_POTENTIAL_TIME_EPSILON);
    (forward - backward) / (2.0 * VECTOR_POTENTIAL_TIME_EPSILON)
}

fn line_integral_component(
    start: Point,
    axis: Vector3<f64>,
    axis_index: usize,
    distance: f64,
    residual: &impl Fn(Point) -> Vector3<f64>,
    geometry: &CoordSampleGeometry,
) -> f64 {
    if distance.abs() <= f64::EPSILON {
        return 0.0;
    }

    let step = distance / SCALAR_POTENTIAL_LINE_STEPS as f64;
    (0..SCALAR_POTENTIAL_LINE_STEPS)
        .map(|index| {
            let offset = (index as f64 + 0.5) * step;
            let point = Point {
                x: start.x + axis.x * offset,
                y: start.y + axis.y * offset,
                z: start.z + axis.z * offset,
            };
            let scale = geometry
                .axis_scale(Vector3::new(point.x, point.y, point.z), axis_index)
                .unwrap_or(0.0);
            residual(point).dot(&axis) * scale * step
        })
        .sum()
}
