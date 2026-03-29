use crate::maths::differential::Form;
use crate::maths::space::Space;
use crate::maths::{expr_to_fastexpr3d, FastExpr3d, Point};
use crate::toolbox::logging::LOGGER;
use std::ops::Mul;

#[derive(Clone)]
pub struct VectorField {
    dual_expr: Form,
    otn_expr: Form,
    fast_otn_expr: [FastExpr3d; 3],
}

impl VectorField {
    pub fn new(expr: Form, space: &Space) -> Self {
        if expr.n_forms() != 1 && expr.n_forms() != 2 {
            LOGGER.error("Vector field must have 1 or 2 forms");
        }
        let dual_expr = expr;
        let otn_expr = dual_expr.to_otn_base(space);
        let fast_otn_expr = Self::compile_fast_otn_expr(&otn_expr);
        Self {
            dual_expr,
            otn_expr,
            fast_otn_expr,
        }
    }

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
        let fast_otn_expr = Self::compile_fast_otn_expr(&otn_expr);
        Self {
            dual_expr,
            otn_expr,
            fast_otn_expr,
        }
    }

    fn compile_fast_otn_expr(otn_expr: &Form) -> [FastExpr3d; 3] {
        [
            expr_to_fastexpr3d(otn_expr.get_expr(0).clone()),
            expr_to_fastexpr3d(otn_expr.get_expr(1).clone()),
            expr_to_fastexpr3d(otn_expr.get_expr(2).clone()),
        ]
    }

    pub fn at(&self, point: Point) -> Point {
        Point {
            x: self.fast_otn_expr[0](point.x, point.y, point.z),
            y: self.fast_otn_expr[1](point.x, point.y, point.z),
            z: self.fast_otn_expr[2](point.x, point.y, point.z),
        }
    }

    pub fn get_dual(&self) -> &Form {
        &self.dual_expr
    }
}
