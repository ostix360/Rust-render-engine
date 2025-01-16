#![allow(unused)]

use exmex::FloatOpsFactory;
use exmex::prelude::*;
use crate::maths::{Expr, ExternalDerivative, Hodge};

pub struct DifferentialForm {
    n_form: Vec<u8>,
    expr: Expr,
    dim: u8,
}

impl Hodge for DifferentialForm {
    fn hodge_star(&self) -> Self {
        todo!()
    }
}

impl ExternalDerivative for DifferentialForm {
    fn d(&self) -> Self {
        todo!()
    }
}
