//! Scalar and vector field runtime representations.
//!
//! Field values are stored symbolically for basis conversion and compiled into numeric closures
//! for per-frame sampling. Vector fields keep both dual-basis and orthonormal-tangent-basis
//! representations because geometric tangent view, dual tangent view, gradient, and curl render
//! paths need different bases.

use crate::maths::differential::{Form, FormBasis};
use crate::maths::space::Space;
use crate::maths::{derivate, expr_to_fastexpr3d, Expr, ExternalDerivative, FastExpr3d, Point};
use crate::toolbox::logging::LOGGER;

#[derive(Clone)]
pub struct ScalarField {
    expr: Expr,
    fast_expr: FastExpr3d,
}

impl ScalarField {
    /// Builds a scalar field from one symbolic expression.
    pub fn new(expr: Expr) -> Self {
        let fast_expr = expr_to_fastexpr3d(expr.clone());
        Self { expr, fast_expr }
    }

    /// Evaluates the scalar field at one abstract coordinate.
    pub fn at(&self, point: Point) -> f64 {
        (self.fast_expr)(point.x, point.y, point.z)
    }

    /// Returns the stored symbolic expression.
    pub fn get_expr(&self) -> &Expr {
        &self.expr
    }
}

#[derive(Clone)]
pub struct VectorField {
    dual_expr: Form,
    otn_expr: Form,
    fast_dual_expr: [FastExpr3d; 3],
    fast_otn_expr: [FastExpr3d; 3],
    fast_otn_jacobian: [[FastExpr3d; 3]; 3],
}

impl VectorField {
    /// Builds a vector field from expressions already expressed in the dual basis.
    ///
    /// The field caches both dual and orthonormal-tangent representations together with the
    /// Jacobian needed for local linearization.
    pub fn new(expr: Form, space: &Space) -> Self {
        if expr.n_forms() != 1 {
            LOGGER.error("Vector field must be built from a 1-form");
        }
        if expr.basis() != FormBasis::Natural {
            panic!("VectorField::new expects a natural-basis 1-form");
        }
        let dual_expr = expr;
        let otn_expr = dual_expr.to_otn_base(space);
        let fast_dual_expr = Self::compile_fast_expr(&dual_expr);
        let fast_otn_expr = Self::compile_fast_otn_expr(&otn_expr);
        let fast_otn_jacobian = Self::compile_fast_otn_jacobian(&otn_expr);
        Self {
            dual_expr,
            otn_expr,
            fast_dual_expr,
            fast_otn_expr,
            fast_otn_jacobian,
        }
    }

    /// Builds a vector field from expressions expressed in the orthonormal tangent basis.
    ///
    /// The dual representation is derived immediately so both bases stay available for later
    /// evaluation.
    pub fn from_otn(expr: Form, space: &Space) -> Self {
        if expr.n_forms() != 1 {
            LOGGER.error(
                format!(
                    "Vector field must be built from a 1-form but got {}",
                    expr.n_forms()
                )
                .as_str(),
            );
        }
        if expr.basis() != FormBasis::Orthonormal {
            panic!("VectorField::from_otn expects an orthonormal-basis 1-form");
        }
        let otn_expr = expr;
        let dual_expr = otn_expr.to_dual_base(space);
        let fast_dual_expr = Self::compile_fast_expr(&dual_expr);
        let fast_otn_expr = Self::compile_fast_otn_expr(&otn_expr);
        let fast_otn_jacobian = Self::compile_fast_otn_jacobian(&otn_expr);
        Self {
            dual_expr,
            otn_expr,
            fast_dual_expr,
            fast_otn_expr,
            fast_otn_jacobian,
        }
    }

    /// Builds the gradient field associated with one scalar expression.
    pub fn gradient_from_scalar(expr: Expr, space: &Space) -> Self {
        let mut scalar_form = Form::new(vec![expr], 0);
        Self::new(scalar_form.d(), space)
    }

    /// Builds the rendered curl field associated with one vector input expressed in the OTN basis.
    pub fn curl_from_otn(expr: Form, space: &Space) -> Self {
        if expr.n_forms() != 1 {
            LOGGER.error("Curl input must be a 1-form");
        }

        let mut dual_input = expr.to_dual_base(space);
        let curl_dual = dual_input.d();
        let curl_otn = curl_dual.to_otn_base(space).hodge_star_otn_3d();
        Self::from_otn(curl_otn, space)
    }

    /// Compiles the three components of a form into numeric closures.
    ///
    /// The components are assumed to follow the axis ordering already enforced by `Form`.
    fn compile_fast_expr(expr: &Form) -> [FastExpr3d; 3] {
        [
            expr_to_fastexpr3d(expr.get_expr(0).clone()),
            expr_to_fastexpr3d(expr.get_expr(1).clone()),
            expr_to_fastexpr3d(expr.get_expr(2).clone()),
        ]
    }

    /// Compiles the orthonormal-tangent components of the field into numeric closures.
    ///
    /// This is a thin wrapper that exists to keep basis-specific call sites explicit.
    fn compile_fast_otn_expr(otn_expr: &Form) -> [FastExpr3d; 3] {
        Self::compile_fast_expr(otn_expr)
    }

    /// Compiles the Jacobian of the orthonormal-tangent field components.
    ///
    /// Each entry stores one partial derivative needed by the local linear approximation used
    /// in tangent mode.
    fn compile_fast_otn_jacobian(otn_expr: &Form) -> [[FastExpr3d; 3]; 3] {
        let x = "x".to_string();
        let y = "y".to_string();
        let z = "z".to_string();

        [
            [
                expr_to_fastexpr3d(derivate(otn_expr.get_expr(0).clone(), &x)),
                expr_to_fastexpr3d(derivate(otn_expr.get_expr(0).clone(), &y)),
                expr_to_fastexpr3d(derivate(otn_expr.get_expr(0).clone(), &z)),
            ],
            [
                expr_to_fastexpr3d(derivate(otn_expr.get_expr(1).clone(), &x)),
                expr_to_fastexpr3d(derivate(otn_expr.get_expr(1).clone(), &y)),
                expr_to_fastexpr3d(derivate(otn_expr.get_expr(1).clone(), &z)),
            ],
            [
                expr_to_fastexpr3d(derivate(otn_expr.get_expr(2).clone(), &x)),
                expr_to_fastexpr3d(derivate(otn_expr.get_expr(2).clone(), &y)),
                expr_to_fastexpr3d(derivate(otn_expr.get_expr(2).clone(), &z)),
            ],
        ]
    }

    /// Evaluates the field components in the orthonormal tangent basis at one point.
    ///
    /// The returned `Point` contains the x, y, and z components in abstract coordinate order.
    pub fn at(&self, point: Point) -> Point {
        Point {
            x: self.fast_otn_expr[0](point.x, point.y, point.z),
            y: self.fast_otn_expr[1](point.x, point.y, point.z),
            z: self.fast_otn_expr[2](point.x, point.y, point.z),
        }
    }

    /// Evaluates the field components in the dual basis at one point.
    ///
    /// This is used when building dual tangent overlays and other covector-oriented views.
    pub fn dual_at(&self, point: Point) -> Point {
        Point {
            x: self.fast_dual_expr[0](point.x, point.y, point.z),
            y: self.fast_dual_expr[1](point.x, point.y, point.z),
            z: self.fast_dual_expr[2](point.x, point.y, point.z),
        }
    }

    /// Evaluates the first-order Taylor approximation of the field around an anchor point.
    ///
    /// The `delta` argument is interpreted as an abstract-space offset from `anchor`.
    pub fn linearized_at(&self, anchor: Point, delta: Point) -> Point {
        let anchor_value = self.at(anchor);

        let dot_row = |row: &[FastExpr3d; 3]| {
            row[0](anchor.x, anchor.y, anchor.z) * delta.x
                + row[1](anchor.x, anchor.y, anchor.z) * delta.y
                + row[2](anchor.x, anchor.y, anchor.z) * delta.z
        };

        Point {
            x: anchor_value.x + dot_row(&self.fast_otn_jacobian[0]),
            y: anchor_value.y + dot_row(&self.fast_otn_jacobian[1]),
            z: anchor_value.z + dot_row(&self.fast_otn_jacobian[2]),
        }
    }

    /// Returns the symbolic field representation stored in the dual basis.
    ///
    /// Callers can use this for further symbolic manipulation or debugging.
    pub fn get_dual(&self) -> &Form {
        &self.dual_expr
    }
}
