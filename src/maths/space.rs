use exmex::{Calculate, Differentiate, ExError, ExResult, Express};
use nalgebra::{Dim, Matrix, Matrix1, Matrix3, MatrixXx4, VecStorage};
use crate::maths::{Expr, ExternalDerivative, COORD};
use crate::maths::differential::Form;
use crate::toolbox::logging::LOGGER;

pub type Metric = Matrix3<Expr>;

struct Space {
    dim: u32,
    metric: Metric
}

impl Space {

    pub fn new(x_eq: Expr, y_eq: Expr, z_eq: Expr) -> Space {
        // For now, we only support 3D spaces with Euclidean metric
        let two = Expr::from_num(2f64);
        let x_form = Form::new(vec![x_eq], 0).d();
        let y_form = Form::new(vec![y_eq], 0).d();
        let z_form = Form::new(vec![z_eq], 0).d();
        
        todo!()
    }
}
