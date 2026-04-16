//! Coordinate-system embedding, curvature estimation, and tangent-basis evaluation.

use crate::maths::space::Space;
use crate::maths::{
    derivate, expr_to_fastexpr2dto1d, expr_to_fastexpr3d, Expr, FastExpr2dto1d, FastExpr3d,
};
use integrate::prelude::trapezoidal_rule;
use mathhook::prelude::*;
use mathhook::Symbol;
use nalgebra::{vector, Vector3};
use std::ops::{Add, Deref};

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
    ///
    /// When the sampled derivative is numerically degenerate, the provided fallback basis
    /// vector is returned instead.
    fn eval_compiled_axis(
        point: Vector3<f64>,
        axis: &[FastExpr3d; 3],
        fallback: Vector3<f64>,
    ) -> Vector3<f64> {
        let tangent = vector![
            axis[0](point.x, point.y, point.z),
            axis[1](point.x, point.y, point.z),
            axis[2](point.x, point.y, point.z)
        ];
        if !tangent.x.is_finite()
            || !tangent.y.is_finite()
            || !tangent.z.is_finite()
            || tangent.norm() <= 1e-9
        {
            fallback
        } else {
            tangent.normalize()
        }
    }

    /// Evaluates the three normalized tangent directions at an abstract point.
    ///
    /// Each basis vector is derived from the compiled partial derivatives of the embedding and
    /// falls back to the canonical axes when needed.
    pub fn eval_tangent_basis(&self, point: Vector3<f64>) -> [Vector3<f64>; 3] {
        [
            Self::eval_compiled_axis(point, &self.tangent_x, vector![1.0, 0.0, 0.0]),
            Self::eval_compiled_axis(point, &self.tangent_y, vector![0.0, 1.0, 0.0]),
            Self::eval_compiled_axis(point, &self.tangent_z, vector![0.0, 0.0, 1.0]),
        ]
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
