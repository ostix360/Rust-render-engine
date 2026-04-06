#![allow(unused)]

use crate::maths::space::{Metric, Space};
use crate::maths::{derivate, Expr, ExternalDerivative, Hodge};
use crate::toolbox::logging::LOGGER;
use mathhook::prelude::expr;
use mathhook_core::matrices::{Matrix, MatrixOperations};
use mathhook_core::Expression;
use std::ops::{Add, Mul, Sub};

/// Conventions:
/// exprs contains the expression in front of each dx, dx^dy ... and only one expr if it's 0-form or 3-form
/// if it's a 1-form the order is dx, dy, dz
/// if it's a 2-form the order is dx^dy, dy^dz, dz^dx

#[derive(Clone)]
pub struct Form {
    pub exprs: Vec<Expr>,
    n_forms: usize,
}

impl Form {
    pub fn new(exprs: Vec<Expr>, n_forms: usize) -> Self {
        Self { exprs, n_forms }
    }

    /// This square function has a sense only for a 1-form and calculates the square of the 1 form treated like a simple expression
    /// Return a vec of expr containing is this order dx^2 dxdy dy^2 dxdz dydz dz^2
    pub fn square(&self) -> Vec<Expr> {
        if self.n_forms != 1 {
            panic!("Square only works for 1-form")
        }
        let mut out = Vec::with_capacity(6);
        let two = expr!(2);
        out.push(Expr::pow(self.exprs[0].clone(), two.clone()));
        out.push(
            self.exprs[0]
                .clone()
                .mul(self.exprs[1].clone())
                .mul(two.clone()),
        );
        out.push(Expr::pow(self.exprs[1].clone(), two.clone()));
        out.push(
            self.exprs[0]
                .clone()
                .mul(self.exprs[2].clone())
                .mul(two.clone()),
        );
        out.push(
            self.exprs[1]
                .clone()
                .mul(self.exprs[2].clone())
                .mul(two.clone()),
        );
        out.push(Expr::pow(self.exprs[2].clone(), two.clone()));
        out
    }

    pub fn to_otn_base(&self, space: &Space) -> Form {
        match self.n_forms {
            0 => Form::new(vec![self.exprs[0].clone()], 0),
            1 => {
                let nat_to_otn = space.natural_to_otn();
                let mut new_exprs = Vec::with_capacity(3);
                for row in 0..3 {
                    let transformed = self.exprs[0]
                        .clone()
                        .mul(nat_to_otn.get_element(row, 0))
                        .add(self.exprs[1].clone().mul(nat_to_otn.get_element(row, 1)))
                        .add(self.exprs[2].clone().mul(nat_to_otn.get_element(row, 2)));
                    new_exprs.push(transformed);
                }
                Form::new(new_exprs, 1)
            }
            2 => {
                // TODO make compatible with non OTN metrics
                let mut new_exprs = Vec::with_capacity(3);
                let nat_to_otn = space.natural_to_otn();
                new_exprs.push(
                    self.exprs[0]
                        .clone()
                        .mul(nat_to_otn.get_element(0, 0))
                        .mul(nat_to_otn.get_element(1, 0)),
                );
                new_exprs.push(
                    self.exprs[1]
                        .clone()
                        .mul(nat_to_otn.get_element(1, 0))
                        .mul(nat_to_otn.get_element(2, 0)),
                );
                new_exprs.push(
                    self.exprs[2]
                        .clone()
                        .mul(nat_to_otn.get_element(2, 0))
                        .mul(nat_to_otn.get_element(0, 0)),
                );
                Form::new(new_exprs, 2)
            }
            3 => {
                let nat_to_otn = space.natural_to_otn();
                let new_expr = self.exprs[0]
                    .clone()
                    .mul(nat_to_otn.get_element(0, 0))
                    .mul(nat_to_otn.get_element(1, 0))
                    .mul(nat_to_otn.get_element(2, 0));
                Form::new(vec![new_expr], 3)
            }
            _ => panic!("Unknown number of forms {}", self.n_forms),
        }
    }

    pub fn to_dual_base(&self, space: &Space) -> Form {
        match self.n_forms {
            0 => Form::new(vec![self.exprs[0].clone()], 0),
            1 => {
                let otn_to_nat = space.otn_to_natural();
                let mut new_exprs = Vec::with_capacity(3);
                for row in 0..3 {
                    let transformed = self.exprs[0]
                        .clone()
                        .mul(otn_to_nat.get_element(row, 0))
                        .add(self.exprs[1].clone().mul(otn_to_nat.get_element(row, 1)))
                        .add(self.exprs[2].clone().mul(otn_to_nat.get_element(row, 2)));
                    new_exprs.push(transformed);
                }
                Form::new(new_exprs, 1)
            }
            2 => {
                let mut new_exprs = Vec::with_capacity(3);
                let otn_to_nat = space.otn_to_natural();
                new_exprs.push(
                    self.exprs[0]
                        .clone()
                        .mul(otn_to_nat.get_element(0, 0))
                        .mul(otn_to_nat.get_element(1, 0)),
                );
                new_exprs.push(
                    self.exprs[1]
                        .clone()
                        .mul(otn_to_nat.get_element(1, 0))
                        .mul(otn_to_nat.get_element(2, 0)),
                );
                new_exprs.push(
                    self.exprs[2]
                        .clone()
                        .mul(otn_to_nat.get_element(2, 0))
                        .mul(otn_to_nat.get_element(0, 0)),
                );
                Form::new(new_exprs, 2)
            }
            3 => {
                let otn_to_nat = space.otn_to_natural();
                let new_expr = self.exprs[0]
                    .clone()
                    .mul(otn_to_nat.get_element(0, 0))
                    .mul(otn_to_nat.get_element(1, 0))
                    .mul(otn_to_nat.get_element(2, 0));
                Form::new(vec![new_expr], 3)
            }
            _ => panic!("Unknown number of forms {}", self.n_forms),
        }
    }

    pub fn to_vec(&self) -> Expression {
        let mut vec = Vec::new();
        vec.push(self.exprs.clone());
        Expression::matrix(vec)
    }

    pub fn n_forms(&self) -> usize {
        self.n_forms
    }

    pub fn get_expr(&self, i: usize) -> &Expr {
        &self.exprs[i]
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
        } else if self.n_forms == 1 {
            let dx_dy = derivate(self.exprs[1].clone(), &"x".to_string())
                .sub(derivate(self.exprs[0].clone(), &"y".to_string()));
            let dy_dz = derivate(self.exprs[2].clone(), &"y".to_string())
                .sub(derivate(self.exprs[1].clone(), &"z".to_string()));
            let dz_dx = derivate(self.exprs[0].clone(), &"z".to_string())
                .sub(derivate(self.exprs[2].clone(), &"x".to_string()));
            Form::new(vec![dx_dy, dy_dz, dz_dx], 2)
        } else if self.n_forms == 2 {
            let dz = derivate(self.exprs[0].clone(), &"z".to_string());
            let dx = derivate(self.exprs[1].clone(), &"x".to_string());
            let dy = derivate(self.exprs[2].clone(), &"y".to_string());
            let dx_dy_dz = dx.add(dy).add(dz);
            Form::new(vec![dx_dy_dz], 3)
        } else if self.n_forms == 3 {
            Form::new(vec![Expr::number(0.)], 0) // zero form
        } else {
            panic!("Unknown number of forms {}", self.n_forms)
        }
    }
}
