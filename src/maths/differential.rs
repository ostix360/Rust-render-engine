#![allow(unused)]

use crate::maths::space::{Metric, Space};
use crate::maths::{derivate, Expr, ExternalDerivative, Hodge};
use crate::toolbox::logging::LOGGER;
use mathhook::prelude::expr;
use mathhook_core::matrices::{Matrix, MatrixOperations};
use mathhook_core::{Expression, Simplify};
use std::ops::{Add, Mul, Sub};

/// Conventions:
/// exprs contains the expression in front of each dx, dx^dy ... and only one expr if it's 0-form or 3-form
/// if it's a 1-form the order is dx, dy, dz
/// if it's a 2-form the order is dx^dy, dy^dz, dz^dx

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormBasis {
    Natural,
    Orthonormal,
}

#[derive(Clone)]
pub struct Form {
    pub exprs: Vec<Expr>,
    n_forms: usize,
    basis: FormBasis,
}

impl Form {
    /// Builds a differential form in the natural coordinate coframe from component expressions.
    ///
    /// Component ordering follows the conventions documented at the top of this module.
    pub fn new(exprs: Vec<Expr>, n_forms: usize) -> Self {
        Self::new_in_basis(exprs, n_forms, FormBasis::Natural)
    }

    /// Builds a differential form in the local orthonormal coframe.
    pub fn new_otn(exprs: Vec<Expr>, n_forms: usize) -> Self {
        Self::new_in_basis(exprs, n_forms, FormBasis::Orthonormal)
    }

    /// Builds a differential form in the supplied basis.
    pub fn new_in_basis(exprs: Vec<Expr>, n_forms: usize, basis: FormBasis) -> Self {
        Self {
            exprs,
            n_forms,
            basis,
        }
    }

    /// Returns the coframe basis currently attached to this form.
    pub fn basis(&self) -> FormBasis {
        self.basis
    }

    /// Returns a clone of this form tagged with another basis.
    pub fn with_basis(mut self, basis: FormBasis) -> Self {
        self.basis = basis;
        self
    }

    /// Panics if this form is not expressed in the expected coframe.
    fn expect_basis(&self, expected: FormBasis, operation: &str) {
        if self.basis != expected {
            panic!(
                "{operation} expects a {:?} form, got {:?}",
                expected, self.basis
            );
        }
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

    /// Transforms a 2-form using the supplied one-form basis conversion matrix.
    /// The matrix is interpreted as mapping old one-form components into the target basis.
    /// The resulting antisymmetric 2-form is then projected back into the stored
    /// `[dx^dy, dy^dz, dz^dx]` ordering.
    fn transform_two_form(&self, transform: &Matrix, target_basis: FormBasis) -> Form {
        if self.n_forms != 2 {
            panic!("2-form transform only works for 2-forms");
        }

        let minor = |row_a: usize, row_b: usize, col_i: usize, col_j: usize| {
            transform
                .get_element(row_a, col_i)
                .mul(transform.get_element(row_b, col_j))
                .sub(
                    transform
                        .get_element(row_b, col_i)
                        .mul(transform.get_element(row_a, col_j)),
                )
        };

        let transformed_xy = self.exprs[0]
            .clone()
            .mul(minor(0, 1, 0, 1))
            .add(self.exprs[1].clone().mul(minor(0, 1, 1, 2)))
            .add(self.exprs[2].clone().mul(minor(0, 1, 2, 0)))
            .simplify();
        let transformed_yz = self.exprs[0]
            .clone()
            .mul(minor(1, 2, 0, 1))
            .add(self.exprs[1].clone().mul(minor(1, 2, 1, 2)))
            .add(self.exprs[2].clone().mul(minor(1, 2, 2, 0)))
            .simplify();
        let transformed_zx = self.exprs[0]
            .clone()
            .mul(minor(2, 0, 0, 1))
            .add(self.exprs[1].clone().mul(minor(2, 0, 1, 2)))
            .add(self.exprs[2].clone().mul(minor(2, 0, 2, 0)))
            .simplify();

        Form::new_in_basis(
            vec![transformed_xy, transformed_yz, transformed_zx],
            2,
            target_basis,
        )
    }

    /// Transforms this form from the natural basis into the orthonormal tangent basis.
    ///
    /// The exact transformation depends on the degree of the form and the vielbein stored in
    /// `Space`.
    pub fn to_otn_base(&self, space: &Space) -> Form {
        self.expect_basis(FormBasis::Natural, "to_otn_base");
        match self.n_forms {
            0 => Form::new_otn(vec![self.exprs[0].clone()], 0),
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
                Form::new_otn(new_exprs, 1)
            }
            2 => {
                let nat_to_otn = space.natural_to_otn();
                self.transform_two_form(&nat_to_otn, FormBasis::Orthonormal)
            }
            3 => {
                let nat_to_otn = space.natural_to_otn();
                let new_expr = self.exprs[0]
                    .clone()
                    .mul(nat_to_otn.get_element(0, 0))
                    .mul(nat_to_otn.get_element(1, 0))
                    .mul(nat_to_otn.get_element(2, 0));
                Form::new_otn(vec![new_expr], 3)
            }
            _ => panic!("Unknown number of forms {}", self.n_forms),
        }
    }

    /// Transforms this form from the orthonormal tangent basis back into the natural dual
    /// basis.
    ///
    /// The transformation mirrors `to_otn_base` using the inverse basis conversion.
    pub fn to_dual_base(&self, space: &Space) -> Form {
        self.expect_basis(FormBasis::Orthonormal, "to_dual_base");
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
                let otn_to_nat = space.otn_to_natural();
                self.transform_two_form(&otn_to_nat, FormBasis::Natural)
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

    /// Packs the form components into a one-row matrix expression.
    ///
    /// This is primarily used when symbolic matrix operations are more convenient than working
    /// with raw vectors.
    pub fn to_vec(&self) -> Expression {
        let mut vec = Vec::new();
        vec.push(self.exprs.clone());
        Expression::matrix(vec)
    }

    /// Returns the degree of this differential form.
    ///
    /// The value ranges from 0 through 3 for the three-dimensional spaces modeled by this
    /// crate.
    pub fn n_forms(&self) -> usize {
        self.n_forms
    }

    /// Returns one component expression from the form.
    ///
    /// The caller is responsible for using an index that matches the component ordering for the
    /// current degree.
    pub fn get_expr(&self, i: usize) -> &Expr {
        &self.exprs[i]
    }

    /// Applies the 3D Hodge star in a positively oriented orthonormal coframe.
    ///
    /// This follows the local OTN rules:
    /// `*1 = e1^e2^e3`, `*e1 = e2^e3`, `*e2 = e3^e1`, `*e3 = e1^e2`,
    /// `*(e1^e2) = e3`, `*(e2^e3) = e1`, `*(e3^e1) = e2`, and
    /// `*(e1^e2^e3) = 1`.
    pub fn hodge_star_otn_3d(&self) -> Form {
        self.expect_basis(FormBasis::Orthonormal, "hodge_star_otn_3d");
        match self.n_forms {
            0 => Form::new_otn(vec![self.exprs[0].clone()], 3),
            1 => Form::new_otn(
                vec![
                    self.exprs[2].clone(),
                    self.exprs[0].clone(),
                    self.exprs[1].clone(),
                ],
                2,
            ),
            2 => Form::new_otn(
                vec![
                    self.exprs[1].clone(),
                    self.exprs[2].clone(),
                    self.exprs[0].clone(),
                ],
                1,
            ),
            3 => Form::new_otn(vec![self.exprs[0].clone()], 0),
            _ => panic!("Unknown number of forms {}", self.n_forms),
        }
    }
}

impl Hodge for Form {
    /// Computes the Hodge dual of this form for the supplied metric.
    ///
    /// This operation is not implemented yet and currently delegates to `todo!()`.
    fn hodge_star(&self, metric: &Metric) -> Form {
        todo!()
    }
}

impl ExternalDerivative for Form {
    /// Computes the exterior derivative of this form.
    ///
    /// The implementation follows the standard 3D coordinate ordering used by the rest of the
    /// symbolic math layer.
    fn d(&mut self) -> Form {
        self.expect_basis(FormBasis::Natural, "d");
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
