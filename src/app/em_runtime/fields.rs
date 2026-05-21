use crate::maths::{expr_to_fastexpr4d, Expr, FastExpr4d, Point};
use nalgebra::Vector3;
use std::sync::Arc;

pub(super) type FastVectorExpr4d = Arc<dyn Fn(f64, f64, f64, f64) -> Vector3<f64> + Send + Sync>;

#[derive(Clone)]
pub(super) struct TimedScalarField {
    fast_expr: FastExpr4d,
}

impl TimedScalarField {
    pub(super) fn new(expr: Expr) -> Self {
        Self {
            fast_expr: expr_to_fastexpr4d(expr),
        }
    }

    pub(super) fn from_fast_expr(fast_expr: FastExpr4d) -> Self {
        Self { fast_expr }
    }

    pub(super) fn at(&self, point: Point, time: f64) -> f64 {
        (self.fast_expr)(point.x, point.y, point.z, time)
    }
}

#[derive(Clone)]
pub(super) struct TimedVectorField {
    fast_exprs: [FastExpr4d; 3],
    vector_expr: Option<FastVectorExpr4d>,
}

impl TimedVectorField {
    pub(super) fn from_exprs(exprs: [Expr; 3]) -> Self {
        Self {
            fast_exprs: [
                expr_to_fastexpr4d(exprs[0].clone()),
                expr_to_fastexpr4d(exprs[1].clone()),
                expr_to_fastexpr4d(exprs[2].clone()),
            ],
            vector_expr: None,
        }
    }

    #[cfg(test)]
    pub(super) fn from_fast_exprs(fast_exprs: [FastExpr4d; 3]) -> Self {
        Self {
            fast_exprs,
            vector_expr: None,
        }
    }

    pub(super) fn from_vector_expr(vector_expr: FastVectorExpr4d) -> Self {
        let x_eval = vector_expr.clone();
        let y_eval = vector_expr.clone();
        let z_eval = vector_expr.clone();
        Self {
            fast_exprs: [
                Arc::new(move |x, y, z, t| x_eval(x, y, z, t).x),
                Arc::new(move |x, y, z, t| y_eval(x, y, z, t).y),
                Arc::new(move |x, y, z, t| z_eval(x, y, z, t).z),
            ],
            vector_expr: Some(vector_expr),
        }
    }

    pub(super) fn at(&self, point: Point, time: f64) -> Vector3<f64> {
        if let Some(vector_expr) = &self.vector_expr {
            return vector_expr(point.x, point.y, point.z, time);
        }

        Vector3::new(
            (self.fast_exprs[0])(point.x, point.y, point.z, time),
            (self.fast_exprs[1])(point.x, point.y, point.z, time),
            (self.fast_exprs[2])(point.x, point.y, point.z, time),
        )
    }
}
