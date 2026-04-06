use crate::maths::differential::Form;
use crate::maths::{Expr, ExternalDerivative};
use mathhook_core::matrices::{Matrix, MatrixOperations};
use mathhook_core::{expr, Expression, Simplify};
use std::ops::{Add, Mul};
use std::sync::Arc;

pub type Metric = Matrix;

pub struct Space {
    #[allow(dead_code)]
    dim: u32,
    metric: Metric,
    vielbein: Expr,
    vielbein_inv: Expr,
}

fn sum3(a: &Expr, b: &Expr, c: &Expr) -> Expr {
    a.add(b).add(c).simplify()
}

#[allow(dead_code)]
fn dot3(a0: &Expr, a1: &Expr, a2: &Expr, b0: &Expr, b1: &Expr, b2: &Expr) -> Expr {
    sum3(
        &(a0.clone().mul(b0.clone())),
        &(a1.clone().mul(b1.clone())),
        &(a2.clone().mul(b2.clone())),
    )
}

impl Space {
    pub fn new(x_eq: Expr, y_eq: Expr, z_eq: Expr) -> Space {
        // J columns: d(X,Y,Z)/d(x), d(X,Y,Z)/d(y), d(X,Y,Z)/d(z)
        let d_x = Form::new(vec![x_eq], 0).d().square();
        let d_y = Form::new(vec![y_eq], 0).d().square();
        let d_z = Form::new(vec![z_eq], 0).d().square();

        // assert!(
        //     d_x.len() == 3 && d_y.len() == 3 && d_z.len() == 3,
        //     "Expected 3 derivatives per coordinate expression; got X={}, Y={}, Z={}",
        //     d_x.len(),
        //     d_y.len(),
        //     d_z.len()
        // );

        let g_xx = sum3(&d_x[0], &d_y[0], &d_z[0]);
        let g_xy = sum3(&d_x[1], &d_y[1], &d_z[1]);
        let g_yy = sum3(&d_x[2], &d_y[2], &d_z[2]);
        let g_xz = sum3(&d_x[3], &d_y[3], &d_z[3]);
        let g_yz = sum3(&d_x[4], &d_y[4], &d_z[4]);
        let g_zz = sum3(&d_x[5], &d_y[5], &d_z[5]);

        let metric: Matrix = Matrix::symmetric(3, vec![g_xx, g_xy, g_yy, g_xz, g_yz, g_zz]);
        let vielbein = Expression::Matrix(Arc::new(metric.cholesky_decomposition().unwrap().l));
        Space {
            dim: 3,
            metric,
            vielbein: vielbein.clone(),
            vielbein_inv: vielbein.inverse(),
        }
    }

    pub fn natural_to_otn(&self) -> Matrix {
        self.vielbein_inv.as_matrix().unwrap().clone()
    }

    pub fn otn_to_natural(&self) -> Matrix {
        self.vielbein.as_matrix().unwrap().clone()
    }

    pub fn get_metric(&self) -> &Metric {
        &self.metric
    }

    #[allow(dead_code)]
    pub fn get_vielbein(&self) -> &Expr {
        &self.vielbein
    }
}
