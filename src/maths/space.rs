use crate::maths::differential::Form;
use crate::maths::{Expr, ExternalDerivative};
use exmex::{Calculate, Express};

pub type Metric = [[Expr; 3]; 3];

pub struct Space {
    dim: u32,
    metric: Metric,
}

fn binary(lhs: &Expr, rhs: &Expr, op: &str) -> Expr {
    let mut out = lhs.clone().operate_binary(rhs.clone(), op).unwrap();
    out.compile();
    out
}

fn sum3(a: &Expr, b: &Expr, c: &Expr) -> Expr {
    let ab = binary(a, b, "+");
    binary(&ab, c, "+")
}

fn dot3(a0: &Expr, a1: &Expr, a2: &Expr, b0: &Expr, b1: &Expr, b2: &Expr) -> Expr {
    let p0 = binary(a0, b0, "*");
    let p1 = binary(a1, b1, "*");
    let p2 = binary(a2, b2, "*");
    sum3(&p0, &p1, &p2)
}

impl Space {
    pub fn new(x_eq: Expr, y_eq: Expr, z_eq: Expr) -> Space {
        // J columns: d(X,Y,Z)/d(x), d(X,Y,Z)/d(y), d(X,Y,Z)/d(z)
        let d_x = Form::new(vec![x_eq], 0).d().exprs;
        let d_y = Form::new(vec![y_eq], 0).d().exprs;
        let d_z = Form::new(vec![z_eq], 0).d().exprs;

        assert!(
            d_x.len() == 3 && d_y.len() == 3 && d_z.len() == 3,
            "Expected 3 derivatives per coordinate expression; got X={}, Y={}, Z={}",
            d_x.len(),
            d_y.len(),
            d_z.len()
        );

        let g_xx = dot3(&d_x[0], &d_y[0], &d_z[0], &d_x[0], &d_y[0], &d_z[0]);
        let g_xy = dot3(&d_x[0], &d_y[0], &d_z[0], &d_x[1], &d_y[1], &d_z[1]);
        let g_xz = dot3(&d_x[0], &d_y[0], &d_z[0], &d_x[2], &d_y[2], &d_z[2]);
        let g_yy = dot3(&d_x[1], &d_y[1], &d_z[1], &d_x[1], &d_y[1], &d_z[1]);
        let g_yz = dot3(&d_x[1], &d_y[1], &d_z[1], &d_x[2], &d_y[2], &d_z[2]);
        let g_zz = dot3(&d_x[2], &d_y[2], &d_z[2], &d_x[2], &d_y[2], &d_z[2]);

        let metric: Metric = [
            [g_xx, g_xy.clone(), g_xz.clone()],
            [g_xy, g_yy, g_yz.clone()],
            [g_xz, g_yz, g_zz],
        ];

        Space { dim: 3, metric }
    }

    pub fn get_metric(&self) -> &Metric {
        &self.metric
    }
}
