use nalgebra::{Dim, Matrix, Matrix1, MatrixXx4, VecStorage};
use crate::maths::Expr;
use crate::toolbox::logging::LOGGER;

type Metric<D> = Matrix<Expr, D, D, VecStorage<Expr, D, D>>; // TODO Redefine it 

struct Space<D :Dim> {
    dim: u32,
    metric: Metric<D>
}

impl<D: Dim> Space<D> {

    pub fn new(exp: Vec<Expr>) -> Space<D> {
        todo!();
        if exp.len() >= 4 {
            LOGGER.error("");
            panic!()
        }
        let dim = exp.len() as u8;
        
    }
}
