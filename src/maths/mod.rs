#![allow(unused)]
//! Symbolic math helpers and fast evaluator compilation used by the runtime.

use crate::maths::space::Metric;
use egui::TextBuffer;
use lazy_static::lazy_static;
use mathhook::prelude::Simplify;
use mathhook::Expression;
use mathhook_core::{Derivative, EvalContext, Symbol};
use nalgebra::Vector3;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use typed_floats::NonNaN;

pub mod differential;
pub mod field;
pub mod space;

pub type Expr = Expression;
pub type FastExpr1d = Arc<dyn Fn(f64) -> f64 + Sync>;

pub type FastExpr2dto1d = Arc<dyn Fn(f64, f64) -> FastExpr1d>;
pub type FastExpr3d = Arc<dyn Fn(f64, f64, f64) -> f64 + Sync>;

pub const COORD: [&str; 3] = ["x", "y", "z"];

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Converts a scalar into a mathhook expression node.
///
/// This helper keeps numeric literal construction concise throughout the symbolic math code.
#[inline]
pub fn num(value: f64) -> Expression {
    Expression::from(value)
}

// pub fn integrate1d(f: &FastExpr, interval: (f64, f64)) -> f64 {
//     let (a, b) = interval;
//     let len = b - a;
//
//     // let mut grid = DiscreteGrid::new(
//     //     vec![
//     //         Some(Grid::Continuous(ContinuousGrid::new(1, 128, 2000, None, false)))
//     //     ],
//     //     0.01,
//     //     false,
//     // );
//
//     let mut grid = ContinuousGrid::new(1, 20, 1000, None, false);
//
//     let mut rng = MonteCarloRng::new(0,0,);
//     let mut sample = Sample::new();
//     for it in 1..10 {
//         for _ in 0..500 {
//             grid.sample(&mut rng, &mut sample);
//             // if let Sample::Discrete(_weight, i, cont_sample) = &sample {
//             //     if let Sample::Continuous(_cont_weight, xs ) = cont_sample.as_ref().unwrap().as_ref() {
//             //         grid.add_training_sample(&sample, func(xs[0])).unwrap();
//             //     }
//             // }
//             if let Sample::Continuous(_cont_weight, xs ) = &sample{
//                 grid.add_training_sample(&sample, f(xs[0] * len + a)).unwrap();
//             }
//         }
//         grid.update(1.5);
//     };
//
//     grid.accumulator.avg
// }

/// Compiles a symbolic expression into a single-variable numeric closure.
///
/// The expression is simplified first and must reference exactly one variable.
#[inline]
pub fn expr_to_fastexpr1d(mut expr: Expr) -> FastExpr1d {
    let eval_expr = expr.simplify();
    if eval_expr.find_variables().len() > 1 {
        panic!("Cannot convert expression to fastexpr1d: too many variables");
    }
    let var_name = eval_expr.find_variables()[0].clone();

    let eval = move |x: f64| -> f64 {
        let mut vars = HashMap::new();
        vars.insert(var_name.name().to_string(), num(x));
        eval_expr.substitute(&vars).evaluate_to_f64().unwrap()
    };
    Arc::new(eval)
}

/// Compiles a three-variable expression into a closure that fixes two coordinates and returns a
/// one-dimensional evaluator.
///
/// The `var_name` argument chooses which coordinate remains free in the returned closure.
#[inline]
pub fn expr_to_fastexpr2dto1d(mut expr: Expr, var_name: String) -> FastExpr2dto1d {
    let eval_expr = expr.simplify();
    let eval_expr = Arc::new(eval_expr);
    /// Builds the substitution map for the chosen free coordinate.
    ///
    /// The remaining two expressions are assigned to the other coordinate names in canonical
    /// `x/y/z` order.
    #[inline]
    fn select_arg(
        target: &str,
        value: Expression,
        first: Expression,
        second: Expression,
    ) -> HashMap<String, Expression> {
        let mut map = HashMap::new();
        match (target) {
            "x" => {
                map.insert("x".to_string(), value);
                map.insert("y".to_string(), first);
                map.insert("z".to_string(), second);
            }
            "y" => {
                map.insert("x".to_string(), first);
                map.insert("y".to_string(), value);
                map.insert("z".to_string(), second);
            }
            "z" => {
                map.insert("x".to_string(), first);
                map.insert("y".to_string(), second);
                map.insert("z".to_string(), value);
            }
            _ => panic!("Unknown target {}", target),
        }
        map
    };
    let func = move |x_: f64, y_: f64| -> FastExpr1d {
        let eval_expr = eval_expr.clone();
        let name = var_name.clone();
        let expr1d_func = move |z_: f64| -> f64 {
            let vars = select_arg(&name, num(z_), num(x_), num(y_));
            eval_expr
                .substitute_and_simplify(&vars)
                .evaluate_to_f64()
                .unwrap()
        };
        Arc::new(expr1d_func)
    };
    Arc::new(func)
}

/// Compiles a symbolic expression into a fast three-variable numeric closure.
///
/// The closure evaluates with a numeric context first and falls back to a manual evaluator for
/// functions not handled by the primary path.
pub fn expr_to_fastexpr3d(expr: Expr) -> FastExpr3d {
    let eval_expr = Arc::new(expr.simplify());
    let eval = move |x: f64, y: f64, z: f64| -> f64 {
        let mut vars = HashMap::with_capacity(3);
        vars.insert("x".to_string(), num(x));
        vars.insert("y".to_string(), num(y));
        vars.insert("z".to_string(), num(z));
        let substituted = eval_expr.substitute(&vars);
        eval_expr
            .evaluate_with_context(&EvalContext::numeric(vars))
            .ok()
            .and_then(|value| value.evaluate_to_f64().ok())
            .or_else(|| eval_numeric_fallback(&substituted).ok())
            .filter(|value| value.is_finite())
            .unwrap_or(f64::NAN)
    };
    Arc::new(eval)
}

/// Recursively evaluates an expression node using a small numeric fallback interpreter.
///
/// This path exists to cover common transcendental functions when direct numeric evaluation
/// leaves symbolic structure behind.
///
/// This function will be deleted when the lib will be patched
#[inline]
fn eval_numeric_fallback(expr: &Expr) -> Result<f64, mathhook_core::MathError> {
    match expr {
        Expression::Add(terms) => terms
            .iter()
            .try_fold(0.0, |acc, term| Ok(acc + eval_numeric_fallback(term)?)),
        Expression::Mul(factors) => factors
            .iter()
            .try_fold(1.0, |acc, factor| Ok(acc * eval_numeric_fallback(factor)?)),
        Expression::Pow(base, exp) => {
            Ok(eval_numeric_fallback(base)?.powf(eval_numeric_fallback(exp)?))
        }
        Expression::Function { name, args } => {
            let arg = |idx: usize| eval_numeric_fallback(&args[idx]);
            match name.as_ref() {
                "sqrt" => Ok(arg(0)?.sqrt()),
                "sin" => Ok(arg(0)?.sin()),
                "cos" => Ok(arg(0)?.cos()),
                "tan" => Ok(arg(0)?.tan()),
                "asin" | "arcsin" => Ok(arg(0)?.asin()),
                "acos" | "arccos" => Ok(arg(0)?.acos()),
                "atan" | "arctan" => Ok(arg(0)?.atan()),
                "ln" => Ok(arg(0)?.ln()),
                "log" => Ok(arg(0)?.ln()),
                "exp" => Ok(arg(0)?.exp()),
                _ => expr.evaluate_to_f64(),
            }
        }
        _ => expr.evaluate_to_f64(),
    }
}

/// Differentiates an expression with respect to one named variable and simplifies the result.
///
/// The variable name must match the symbolic names used throughout the coordinate and field
/// code.
#[inline]
pub fn derivate(expr: Expr, variable_name: &String) -> Expr {
    expr.derivative(Symbol::new(variable_name)).simplify()
}

/// Converts a raw vertex triple into a non-NaN vector.
///
/// An error is returned if any component contains `NaN`.
#[inline]
pub fn to_nn_vec(v: [f32; 3]) -> Result<Vector3<NonNaN<f32>>, &'static str> {
    Ok(Vector3::new(
        NonNaN::<f32>::new(v[0]).ok().ok_or("NaN in vertex")?,
        NonNaN::<f32>::new(v[1]).ok().ok_or("NaN in vertex")?,
        NonNaN::<f32>::new(v[2]).ok().ok_or("NaN in vertex")?,
    ))
}

pub trait Hodge {
    /// Computes the Hodge dual of the implementing value with respect to the supplied metric.
    ///
    /// Implementations are expected to preserve the basis conventions used across the `maths`
    /// module.
    fn hodge_star(&self, metric: &Metric) -> Self;
}

pub trait ExternalDerivative {
    /// Computes the exterior derivative of the implementing value.
    ///
    /// Implementations follow the differential-form conventions established in this module.
    fn d(&mut self) -> Self;
}

#[cfg(test)]
mod tests {
    use super::expr_to_fastexpr3d;
    use mathhook_core::Parser;

    #[test]
    fn expr_to_fastexpr3d_returns_nan_on_singular_eval() {
        let expr = Parser::default().parse("1 / x").unwrap();
        let eval = expr_to_fastexpr3d(expr);

        let result = eval(0.0, 0.0, 0.0);

        assert!(result.is_nan());
    }
}
