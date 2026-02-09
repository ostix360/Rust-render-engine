use exmex::{Calculate, Differentiate, ExError, ExResult, Express};
use nalgebra::{Dim, Matrix, Matrix1, Matrix3, MatrixXx4, VecStorage};
use crate::maths::{Expr, ExternalDerivative, COORD};
use crate::maths::differential::Form;
use crate::toolbox::logging::LOGGER;

pub type Metric = [[Expr;3];3];

pub struct Space {
    dim: u32,
    metric: Metric
}
fn sum3(a: &Expr, b: &Expr, c: &Expr) -> Expr {
    let mut out = a.clone()
        .operate_binary(b.clone(), "+").unwrap()
        .operate_binary(c.clone(), "+").unwrap();
    out.compile();
    out
}
impl Space {

    pub fn new(x_eq: Expr, y_eq: Expr, z_eq: Expr) -> Space {
        // For now, we only support 3D spaces with Euclidean metric
        let two = Expr::from_num(2f64);
        let x_form = Form::new(vec![x_eq], 0).d();
        let y_form = Form::new(vec![y_eq], 0).d();
        let z_form = Form::new(vec![z_eq], 0).d();
        let x_exprs = x_form.square();
        let y_exprs = y_form.square();
        let z_exprs = z_form.square();
        let need = 6usize;

        assert!(
            x_exprs.len() >= need && y_exprs.len() >= need && z_exprs.len() >= need,
            "Form::square() must yield {need} terms (3D metric). got x={}, y={}, z={}",
            x_exprs.len(),
            y_exprs.len(),
            z_exprs.len()
        );

        let at = |i: usize| -> Expr {
            let x = x_exprs.get(i).unwrap_or_else(|| panic!("x_exprs[{i}] (len={})", x_exprs.len()));
            let y = y_exprs.get(i).unwrap_or_else(|| panic!("y_exprs[{i}] (len={})", y_exprs.len()));
            let z = z_exprs.get(i).unwrap_or_else(|| panic!("z_exprs[{i}] (len={})", z_exprs.len()));
            sum3(x, y, z)
        };

        let dxdx = at(0);
        let dxdy = at(1);
        let dxdz = at(2);
        let dydy = at(3);
        let dydz = at(4);
        let dzdz = at(5);

        // let dxdx = x_exprs[0].clone().operate_binary(y_exprs[0].clone(), "+").unwrap().operate_binary(z_exprs[0].clone(), "+").unwrap();
        // let dxdy = x_exprs[1].clone().operate_binary(y_exprs[1].clone(), "+").unwrap().operate_binary(z_exprs[1].clone(), "+").unwrap();
        // let dxdz = x_exprs[2].clone().operate_binary(y_exprs[2].clone(), "+").unwrap().operate_binary(z_exprs[2].clone(), "+").unwrap();

        // let dydy = x_exprs[3].clone().operate_binary(y_exprs[3].clone(), "+").unwrap().operate_binary(z_exprs[3].clone(), "+").unwrap();
        // let dydz = x_exprs[4].clone().operate_binary(y_exprs[4].clone(), "+").unwrap().operate_binary(z_exprs[4].clone(), "+").unwrap();
        // let dzdz = x_exprs[5].clone().operate_binary(y_exprs[5].clone(), "+").unwrap().operate_binary(z_exprs[5].clone(), "+").unwrap();
        let dxdy_c = dxdy.clone();

        let dxdz_c = dxdz.clone();
        let dydz_c = dydz.clone();
        let metric: Metric = [
            [dxdx, dxdy_c, dxdz_c],
            [dxdy, dydy, dydz_c],
            [dxdz, dydz, dzdz],
        ];

        Space { dim: 3, metric }
    }

    pub fn get_metric(&self) -> &Metric {
        &self.metric
    }
}
