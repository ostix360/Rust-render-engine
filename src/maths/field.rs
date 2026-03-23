use std::collections::HashMap;
use std::ops::Mul;
use mathhook_core::Expression;
use crate::maths::differential::Form;
use crate::maths::Point;
use crate::maths::space::Space;
use crate::toolbox::logging::LOGGER;

#[derive(Clone)]
pub struct VectorField {
    dual_expr: Form,
    otn_expr: Form,
}

impl VectorField {
    pub fn new(expr: Form, space: &Space) -> Self {
        if expr.n_forms() != 1 && expr.n_forms() != 2 {
            LOGGER.error("Vector field must have 1 or 2 forms");
        }
        let dual_expr = expr;
        let otn_expr = dual_expr.to_otn_base(space);
        Self { dual_expr, otn_expr }
    }

    pub fn from_otn(expr: Form, space: &Space) -> Self {
        if expr.n_forms() != 1 && expr.n_forms() != 2 {
            LOGGER.error(format!("Vector field must have 1 or 2 forms but got {}", expr.n_forms()).as_str());
        }
        let otn_expr = expr;
        let dual_expr = otn_expr.to_dual_base(space);
        Self { dual_expr, otn_expr }
    }

    pub fn at(&self, point: Point) -> Point {
        let vars = HashMap::from([
            ("x".to_string(), Expression::number(point.x)),
            ("y".to_string(), Expression::number(point.y)),
            ("z".to_string(), Expression::number(point.z)),
        ]);
        let vec = &self.otn_expr;
        let vx = vec.get_expr(0).substitute(&vars).evaluate_to_f64().unwrap();
        let vy = vec.get_expr(1).substitute(&vars).evaluate_to_f64().unwrap();
        let vz = vec.get_expr(2).substitute(&vars).evaluate_to_f64().unwrap();
        Point { x: vx, y: vy, z: vz }
    }

    pub fn get_dual(&self) -> &Form {
        &self.dual_expr
    }

}
