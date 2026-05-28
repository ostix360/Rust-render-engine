//! Coordinate-system embedding, curvature estimation, and tangent-basis evaluation.

use crate::maths::space::Space;
use crate::maths::{
    derivate, expr_to_fastexpr2dto1d, expr_to_fastexpr3d, Expr, FastExpr2dto1d, FastExpr3d,
};
use integrate::prelude::trapezoidal_rule;
use mathhook::prelude::*;
use mathhook::Symbol;
use nalgebra::{vector, Matrix3, Vector3};
use std::ops::{Add, Deref};

const CARTESIAN_GEOMETRY_EPSILON: f64 = 1.0e-7;

pub struct CoordsSys {
    #[allow(dead_code)]
    x_eq: Expr,
    #[allow(dead_code)]
    y_eq: Expr,
    #[allow(dead_code)]
    z_eq: Expr,
    fast_x_eq: FastExpr3d,
    fast_y_eq: FastExpr3d,
    fast_z_eq: FastExpr3d,
    tangent_x: [FastExpr3d; 3],
    tangent_y: [FastExpr3d; 3],
    tangent_z: [FastExpr3d; 3],
    x_curvature: FastExpr2dto1d,
    y_curvature: FastExpr2dto1d,
    z_curvature: FastExpr2dto1d,
    space: Space,
}

#[derive(Clone)]
pub struct CoordSampleGeometry {
    fast_x_eq: FastExpr3d,
    fast_y_eq: FastExpr3d,
    fast_z_eq: FastExpr3d,
    tangent_x: [FastExpr3d; 3],
    tangent_y: [FastExpr3d; 3],
    tangent_z: [FastExpr3d; 3],
}

impl CoordsSys {
    /// Builds a coordinate system from three embedding expressions.
    ///
    /// The expressions are compiled into fast evaluators, tangent bases, curvature integrands,
    /// and a `Space` descriptor used by field and tangent-space code.
    pub fn new(x_eq: Expr, y_eq: Expr, z_eq: Expr) -> Self {
        let (x_curvature, y_curvature, z_curvature) =
            Self::calculate_curvature(&x_eq, &y_eq, &z_eq);
        let fast_x_eq = expr_to_fastexpr3d(x_eq.clone());
        let fast_y_eq = expr_to_fastexpr3d(y_eq.clone());
        let fast_z_eq = expr_to_fastexpr3d(z_eq.clone());
        let tangent_x = Self::compile_tangent_axis(&x_eq, &y_eq, &z_eq, "x");
        let tangent_y = Self::compile_tangent_axis(&x_eq, &y_eq, &z_eq, "y");
        let tangent_z = Self::compile_tangent_axis(&x_eq, &y_eq, &z_eq, "z");
        let space = Space::new(x_eq.clone(), y_eq.clone(), z_eq.clone());
        Self {
            x_eq,
            y_eq,
            z_eq,
            fast_x_eq,
            fast_y_eq,
            fast_z_eq,
            tangent_x,
            tangent_y,
            tangent_z,
            x_curvature,
            y_curvature,
            z_curvature,
            space,
        }
    }

    /// Builds one-dimensional curvature integrands for each coordinate axis.
    ///
    /// Each returned closure measures the norm of the second derivative along one abstract axis
    /// while keeping the other two coordinates fixed.
    #[inline]
    fn calculate_curvature(
        x_eq: &Expr,
        y_eq: &Expr,
        z_eq: &Expr,
    ) -> (FastExpr2dto1d, FastExpr2dto1d, FastExpr2dto1d) {
        let mut curvature = Vec::new();
        let x = Symbol::new("x");
        let y = Symbol::new("y");
        let z = Symbol::new("z");
        for x_i in [x, y, z] {
            let ddx_1 = &Expression::pow(x_eq.nth_derivative(x_i.clone(), 2), expr!(2));
            let ddx_2 = &Expression::pow(y_eq.nth_derivative(x_i.clone(), 2), expr!(2));
            let ddx_3 = &Expression::pow(z_eq.nth_derivative(x_i, 2), expr!(2));
            let ddx = ddx_1.add(ddx_2).add(ddx_3);
            let ddx = Expression::sqrt(ddx);
            curvature.push(ddx);
        }
        let [a, b, c] = curvature.try_into().expect("COORD must have 3 elements");
        (
            expr_to_fastexpr2dto1d(a, "x".to_string()),
            expr_to_fastexpr2dto1d(b, "y".to_string()),
            expr_to_fastexpr2dto1d(c, "z".to_string()),
        )
    }

    /// Compiles the partial derivatives that define one tangent-basis axis.
    ///
    /// The resulting closures evaluate the embedded-space direction induced by varying the
    /// named abstract coordinate.
    fn compile_tangent_axis(
        x_eq: &Expr,
        y_eq: &Expr,
        z_eq: &Expr,
        axis_name: &str,
    ) -> [FastExpr3d; 3] {
        let axis_name = axis_name.to_string();
        [
            expr_to_fastexpr3d(derivate(x_eq.clone(), &axis_name)),
            expr_to_fastexpr3d(derivate(y_eq.clone(), &axis_name)),
            expr_to_fastexpr3d(derivate(z_eq.clone(), &axis_name)),
        ]
    }

    /// Approximates curvature around an abstract point for each coordinate axis.
    ///
    /// The method integrates the precompiled curvature integrands over a symmetric interval of
    /// length `len` around the supplied point.
    pub fn get_curvature(&self, point: Vector3<f64>, len: f64) -> (f64, f64, f64) {
        let (x, y, z) = (point.x, point.y, point.z);
        let fx = (self.x_curvature)(y, z);
        let fy = (self.y_curvature)(x, z);
        let fz = (self.z_curvature)(x, y);
        let cx = trapezoidal_rule(fx.deref(), x - len, x + len, 100u32);
        let cy = trapezoidal_rule(fy.deref(), y - len, y + len, 100u32);
        let cz = trapezoidal_rule(fz.deref(), z - len, z + len, 100u32);
        (cx, cy, cz)
    }

    /// Evaluates the embedded coordinate expressions at the supplied abstract coordinates.
    ///
    /// The returned tuple contains the world-space `(x, y, z)` position produced by the
    /// compiled expressions.
    pub fn eval(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let x_ = (self.fast_x_eq)(x, y, z);
        let y_ = (self.fast_y_eq)(x, y, z);
        let z_ = (self.fast_z_eq)(x, y, z);
        (x_, y_, z_)
    }

    /// Evaluates the coordinate system and returns the embedded position as a vector.
    ///
    /// This is a convenience wrapper around `eval` for callers that already work with
    /// `Vector3<f64>` values.
    pub fn eval_position(&self, point: Vector3<f64>) -> Vector3<f64> {
        let (x, y, z) = self.eval(point.x, point.y, point.z);
        vector![x, y, z]
    }

    /// Evaluates one compiled tangent axis and normalizes the result.
    fn eval_raw_axis(point: Vector3<f64>, axis: &[FastExpr3d; 3]) -> Vector3<f64> {
        vector![
            axis[0](point.x, point.y, point.z),
            axis[1](point.x, point.y, point.z),
            axis[2](point.x, point.y, point.z)
        ]
    }

    fn normalize_axis(tangent: Vector3<f64>) -> Option<Vector3<f64>> {
        if !tangent.x.is_finite()
            || !tangent.y.is_finite()
            || !tangent.z.is_finite()
            || tangent.norm() <= 1e-9
        {
            None
        } else {
            Some(tangent.normalize())
        }
    }

    fn eval_regular_axis(point: Vector3<f64>, axis: &[FastExpr3d; 3]) -> Option<Vector3<f64>> {
        Self::normalize_axis(Self::eval_raw_axis(point, axis))
    }

    fn axis_is_zero_at(point: Vector3<f64>, axis: &[FastExpr3d; 3]) -> bool {
        let tangent = Self::eval_raw_axis(point, axis);
        tangent.x.is_finite()
            && tangent.y.is_finite()
            && tangent.z.is_finite()
            && tangent.norm() <= 1e-9
    }

    fn eval_regular_tangent_basis_from_axes(
        point: Vector3<f64>,
        tangent_x: &[FastExpr3d; 3],
        tangent_y: &[FastExpr3d; 3],
        tangent_z: &[FastExpr3d; 3],
    ) -> Option<[Vector3<f64>; 3]> {
        Some([
            Self::eval_regular_axis(point, tangent_x)?,
            Self::eval_regular_axis(point, tangent_y)?,
            Self::eval_regular_axis(point, tangent_z)?,
        ])
    }

    fn axis_is_locally_inactive(point: Vector3<f64>, axis: &[FastExpr3d; 3]) -> bool {
        const STENCIL_STEP: f64 = 1.0e-4;

        [
            point,
            point + vector![STENCIL_STEP, 0.0, 0.0],
            point - vector![STENCIL_STEP, 0.0, 0.0],
            point + vector![0.0, STENCIL_STEP, 0.0],
            point - vector![0.0, STENCIL_STEP, 0.0],
            point + vector![0.0, 0.0, STENCIL_STEP],
            point - vector![0.0, 0.0, STENCIL_STEP],
        ]
        .iter()
        .all(|sample| Self::axis_is_zero_at(*sample, axis))
    }

    fn eval_sample_axis(
        point: Vector3<f64>,
        axis: &[FastExpr3d; 3],
        fallback: Vector3<f64>,
    ) -> Option<Vector3<f64>> {
        Self::eval_regular_axis(point, axis)
            .or_else(|| Self::axis_is_locally_inactive(point, axis).then_some(fallback))
    }

    fn eval_sample_tangent_basis_from_axes(
        point: Vector3<f64>,
        tangent_x: &[FastExpr3d; 3],
        tangent_y: &[FastExpr3d; 3],
        tangent_z: &[FastExpr3d; 3],
    ) -> Option<[Vector3<f64>; 3]> {
        Some([
            Self::eval_sample_axis(point, tangent_x, vector![1.0, 0.0, 0.0])?,
            Self::eval_sample_axis(point, tangent_y, vector![0.0, 1.0, 0.0])?,
            Self::eval_sample_axis(point, tangent_z, vector![0.0, 0.0, 1.0])?,
        ])
    }

    fn raw_tangent_axes_from_axes(
        point: Vector3<f64>,
        tangent_x: &[FastExpr3d; 3],
        tangent_y: &[FastExpr3d; 3],
        tangent_z: &[FastExpr3d; 3],
    ) -> Option<[Vector3<f64>; 3]> {
        let tangent = vector![
            tangent_x[0](point.x, point.y, point.z),
            tangent_x[1](point.x, point.y, point.z),
            tangent_x[2](point.x, point.y, point.z)
        ];
        let tangent_y = vector![
            tangent_y[0](point.x, point.y, point.z),
            tangent_y[1](point.x, point.y, point.z),
            tangent_y[2](point.x, point.y, point.z)
        ];
        let tangent_z = vector![
            tangent_z[0](point.x, point.y, point.z),
            tangent_z[1](point.x, point.y, point.z),
            tangent_z[2](point.x, point.y, point.z)
        ];
        if [tangent, tangent_y, tangent_z]
            .iter()
            .any(|axis| !axis.x.is_finite() || !axis.y.is_finite() || !axis.z.is_finite())
        {
            None
        } else {
            Some([tangent, tangent_y, tangent_z])
        }
    }

    /// Evaluates one compiled tangent axis and normalizes the result.
    ///
    /// When the sampled derivative is numerically degenerate, the provided fallback basis
    /// vector is returned instead.
    fn eval_compiled_axis_with_fallback(
        point: Vector3<f64>,
        axis: &[FastExpr3d; 3],
        fallback: Vector3<f64>,
    ) -> Vector3<f64> {
        Self::eval_regular_axis(point, axis).unwrap_or(fallback)
    }

    /// Evaluates the three normalized tangent directions at an abstract point.
    ///
    /// Each basis vector is derived from the compiled partial derivatives of the embedding and
    /// falls back to the canonical axes when needed.
    pub fn eval_tangent_basis(&self, point: Vector3<f64>) -> [Vector3<f64>; 3] {
        [
            Self::eval_compiled_axis_with_fallback(point, &self.tangent_x, vector![1.0, 0.0, 0.0]),
            Self::eval_compiled_axis_with_fallback(point, &self.tangent_y, vector![0.0, 1.0, 0.0]),
            Self::eval_compiled_axis_with_fallback(point, &self.tangent_z, vector![0.0, 0.0, 1.0]),
        ]
    }

    /// Evaluates the tangent basis only when every coordinate direction is regular.
    ///
    /// Coordinate singularities such as cylindrical `r = 0` or spherical poles do not define a
    /// stable local orthonormal frame. Field arrows sampled there can inherit huge metric factors,
    /// so render caches should skip those points instead of silently substituting fallback axes.
    pub fn eval_regular_tangent_basis(&self, point: Vector3<f64>) -> Option<[Vector3<f64>; 3]> {
        Self::eval_regular_tangent_basis_from_axes(
            point,
            &self.tangent_x,
            &self.tangent_y,
            &self.tangent_z,
        )
    }

    /// Evaluates a basis suitable for cached display samples.
    ///
    /// Truly singular points are still rejected, but coordinate systems with an intentionally
    /// inactive axis, such as a 2D polar embedding with `z = 0`, keep the canonical fallback for
    /// that axis so scalar and vector field samples remain visible.
    pub fn eval_sample_tangent_basis(&self, point: Vector3<f64>) -> Option<[Vector3<f64>; 3]> {
        Self::eval_sample_tangent_basis_from_axes(
            point,
            &self.tangent_x,
            &self.tangent_y,
            &self.tangent_z,
        )
    }

    /// Returns a lightweight clone of the compiled geometry evaluators.
    pub fn sample_geometry(&self) -> CoordSampleGeometry {
        CoordSampleGeometry {
            fast_x_eq: self.fast_x_eq.clone(),
            fast_y_eq: self.fast_y_eq.clone(),
            fast_z_eq: self.fast_z_eq.clone(),
            tangent_x: self.tangent_x.clone(),
            tangent_y: self.tangent_y.clone(),
            tangent_z: self.tangent_z.clone(),
        }
    }

    /// Converts components expressed in the local tangent basis into world-space coordinates.
    ///
    /// The supplied basis is assumed to follow the same axis ordering as `eval_tangent_basis`.
    #[allow(dead_code)]
    pub fn eval_otn_vector_with_basis(
        &self,
        basis: &[Vector3<f64>; 3],
        vector: Vector3<f64>,
    ) -> Vector3<f64> {
        basis[0] * vector.x + basis[1] * vector.y + basis[2] * vector.z
    }

    /// Evaluates an abstract-space vector in world space at the supplied point.
    ///
    /// This first samples the tangent basis at `point` and then expands the vector components
    /// in that basis.
    #[allow(dead_code)]
    pub fn eval_otn_vector(&self, point: Vector3<f64>, vector: Vector3<f64>) -> Vector3<f64> {
        let basis = self.eval_tangent_basis(point);
        self.eval_otn_vector_with_basis(&basis, vector)
    }

    /// Checks whether three serialized equations match the expressions stored in this
    /// coordinate system.
    ///
    /// The comparison uses the current string form of the original expressions and is intended
    /// for cheap UI/config diffing.
    #[allow(dead_code)]
    pub fn is_equivalent(&self, eqs: &[String; 3]) -> bool {
        eqs[0] == self.x_eq.to_string()
            && eqs[1] == self.y_eq.to_string()
            && eqs[2] == self.z_eq.to_string()
    }

    /// Returns the metric-space descriptor derived from this coordinate system.
    ///
    /// The returned `Space` is reused by differential-form and vector-field code.
    pub fn get_space(&self) -> &Space {
        &self.space
    }
}

impl CoordSampleGeometry {
    pub fn is_orthonormal_cartesian(&self) -> bool {
        let samples = [
            vector![0.0, 0.0, 0.0],
            vector![1.0, 0.5, -0.25],
            vector![-0.75, 1.25, 0.5],
            vector![0.25, -1.0, 1.5],
        ];
        let Some(reference_axes) = self.raw_tangent_axes(samples[0]) else {
            return false;
        };
        if !is_right_handed_orthonormal(&reference_axes) {
            return false;
        }
        let reference_origin = self.eval_position(samples[0]);

        samples.iter().all(|point| {
            let Some(axes) = self.raw_tangent_axes(*point) else {
                return false;
            };
            if !axes
                .iter()
                .zip(reference_axes.iter())
                .all(|(axis, reference)| (*axis - *reference).norm() <= CARTESIAN_GEOMETRY_EPSILON)
            {
                return false;
            }

            let expected_position = reference_origin
                + reference_axes[0] * (point.x - samples[0].x)
                + reference_axes[1] * (point.y - samples[0].y)
                + reference_axes[2] * (point.z - samples[0].z);
            (self.eval_position(*point) - expected_position).norm() <= CARTESIAN_GEOMETRY_EPSILON
        })
    }

    pub fn eval_position(&self, point: Vector3<f64>) -> Vector3<f64> {
        vector![
            (self.fast_x_eq)(point.x, point.y, point.z),
            (self.fast_y_eq)(point.x, point.y, point.z),
            (self.fast_z_eq)(point.x, point.y, point.z)
        ]
    }

    pub fn eval_regular_tangent_basis(&self, point: Vector3<f64>) -> Option<[Vector3<f64>; 3]> {
        CoordsSys::eval_regular_tangent_basis_from_axes(
            point,
            &self.tangent_x,
            &self.tangent_y,
            &self.tangent_z,
        )
    }

    pub fn volume_density(&self, point: Vector3<f64>) -> Option<f64> {
        let axes = self.raw_tangent_axes(point)?;
        let density = axes[0].dot(&axes[1].cross(&axes[2])).abs();
        density.is_finite().then_some(density)
    }

    fn raw_tangent_axes(&self, point: Vector3<f64>) -> Option<[Vector3<f64>; 3]> {
        CoordsSys::raw_tangent_axes_from_axes(
            point,
            &self.tangent_x,
            &self.tangent_y,
            &self.tangent_z,
        )
    }

    pub fn vector_to_world(&self, basis: &[Vector3<f64>; 3], vector: Vector3<f64>) -> Vector3<f64> {
        basis[0] * vector.x + basis[1] * vector.y + basis[2] * vector.z
    }

    pub fn world_to_components(
        &self,
        basis: &[Vector3<f64>; 3],
        vector: Vector3<f64>,
    ) -> Vector3<f64> {
        let matrix = Matrix3::from_columns(basis);
        matrix
            .try_inverse()
            .map(|inverse| inverse * vector)
            .unwrap_or_else(Vector3::zeros)
    }
}

fn is_right_handed_orthonormal(axes: &[Vector3<f64>; 3]) -> bool {
    let unit_axes = axes
        .iter()
        .all(|axis| (axis.norm() - 1.0).abs() <= CARTESIAN_GEOMETRY_EPSILON);
    let orthogonal_axes = axes[0].dot(&axes[1]).abs() <= CARTESIAN_GEOMETRY_EPSILON
        && axes[0].dot(&axes[2]).abs() <= CARTESIAN_GEOMETRY_EPSILON
        && axes[1].dot(&axes[2]).abs() <= CARTESIAN_GEOMETRY_EPSILON;
    let right_handed =
        (axes[0].cross(&axes[1]).dot(&axes[2]) - 1.0).abs() <= CARTESIAN_GEOMETRY_EPSILON;

    unit_axes && orthogonal_axes && right_handed
}
