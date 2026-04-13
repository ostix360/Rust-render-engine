use crate::maths::differential::Form;
use crate::maths::space::Space;
use crate::maths::{derivate, expr_to_fastexpr3d, FastExpr3d, Point};
use crate::toolbox::logging::LOGGER;
use std::ops::Mul;

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
        if expr.n_forms() != 1 && expr.n_forms() != 2 {
            LOGGER.error("Vector field must have 1 or 2 forms");
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
        if expr.n_forms() != 1 && expr.n_forms() != 2 {
            LOGGER.error(
                format!(
                    "Vector field must have 1 or 2 forms but got {}",
                    expr.n_forms()
                )
                .as_str(),
            );
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
