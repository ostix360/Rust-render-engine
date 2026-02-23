#![allow(unused)]

use std::ops::{Add, Mul, Sub};
use exmex::{parse, FloatOpsFactory};
use exmex::prelude::*;
use mathhook::prelude::expr;
use crate::maths::{derivate, Expr, ExternalDerivative, Hodge};
use crate::maths::space::Metric;
use crate::toolbox::logging::LOGGER;

/// Conventions:
/// exprs contains the expression in front of each dx, dx^dy ... and only one expr if it's 0-form or 3-form
/// if it's a 1-form the order is dx, dy, dz
/// if it's a 2-form the order is dx^dy, dy^dz, dz^dx

pub struct Form {
    pub exprs: Vec<Expr>,
    n_forms: usize
}

impl Form {
    pub fn new(exprs: Vec<Expr>, n_forms: usize) -> Self {
        Self { exprs, n_forms }
    }

    /// This square function has a sense only for a 1-form and calculates the square of the 1 form treated like a simple expression
    /// Return a vec of expr containing is this order dx^2 dxdy dxdz dy^2 dydz dz^2
    pub fn square(&self) -> Vec<Expr> {
        if self.n_forms != 1 {
            panic!("Square only works for 1-form")
        }
        let mut out = Vec::with_capacity(6);
        let two = expr!(2.0);
        out.push(Expr::pow(self.exprs[0].clone(), two.clone()));
        out.push(self.exprs[0].clone().mul(self.exprs[1].clone()).mul(two.clone()));
        out.push(self.exprs[0].clone().mul(self.exprs[2].clone()).mul(two.clone()));
        out.push(Expr::pow(self.exprs[1].clone(), two.clone()));
        out.push(self.exprs[1].clone().mul(self.exprs[2].clone()).mul(two.clone()));
        out.push(Expr::pow(self.exprs[2].clone(), two.clone()));
        out
    }
}

impl Hodge for Form {
    fn hodge_star(&self, metric: &Metric) -> Form {
        todo!()
    }
}

impl ExternalDerivative for Form {
    fn d(&mut self) -> Form {
        if self.n_forms == 0 {
            let dx = derivate(self.exprs[0].clone(), &"x".to_string());
            let dy = derivate(self.exprs[0].clone(), &"y".to_string());
            let dz = derivate(self.exprs[0].clone(), &"z".to_string());
            Form::new(vec![dx, dy, dz], 1)
        }else if self.n_forms == 1 {
            let dx_dy = derivate(self.exprs[1].clone(), &"x".to_string()).sub(
                derivate(self.exprs[0].clone(), &"y".to_string()));
            let dy_dz = derivate(self.exprs[2].clone(), &"y".to_string()).sub(
                derivate(self.exprs[1].clone(), &"z".to_string()));
            let dz_dx = derivate(self.exprs[0].clone(), &"z".to_string()).sub(
                derivate(self.exprs[2].clone(), &"x".to_string()));
            Form::new(vec![dx_dy, dy_dz, dz_dx], 2)
        }else if self.n_forms == 2 {
            let dz = derivate(self.exprs[0].clone(), &"z".to_string());
            let dx = derivate(self.exprs[1].clone(), &"x".to_string());
            let dy = derivate(self.exprs[2].clone(), &"y".to_string());
            let dx_dy_dz = dx.add(dy).add(dz);
            Form::new(vec![dx_dy_dz], 3)
        }else if self.n_forms == 3 {
            Form::new(vec![Expr::number(0.)], 0) // zero form
        }else {
            panic!("Unknown number of forms {}", self.n_forms)
        }
    }
}
