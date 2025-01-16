#![allow(unused)]

use exmex::{FlatEx, FloatOpsFactory};

mod differential;
mod space;

pub type Expr = FlatEx<f32, FloatOpsFactory<f32>>;

trait Hodge {
    fn hodge_star(&self) -> Self;
}

trait ExternalDerivative {
    fn d(&self) -> Self;
}

