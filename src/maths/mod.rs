#![allow(unused)]

use crate::maths::space::Metric;
use egui::TextBuffer;
use lazy_static::lazy_static;
use mathhook::prelude::Simplify;
use mathhook::Expression;
use mathhook_core::{Derivative, Symbol};
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

pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

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

#[inline]
pub fn expr_to_fastexpr2dto1d(mut expr: Expr, var_name: String) -> FastExpr2dto1d {
    let eval_expr = expr.simplify();
    let eval_expr = Arc::new(eval_expr);
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

pub fn expr_to_fastexpr3d(mut expr: Expr) -> FastExpr3d {
    let eval_expr = expr.simplify();
    let eval = move |x: f64, y: f64, z: f64| -> f64 {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), num(x));
        vars.insert("y".to_string(), num(y));
        vars.insert("z".to_string(), num(z));
        expr.substitute(&vars).evaluate_to_f64().unwrap()
    };
    Arc::new(eval)
}

pub fn derivate(expr: Expr, variable_name: &String) -> Expr {
    expr.derivative(Symbol::new(variable_name)).simplify()
}

pub fn to_nn_vec(v: [f32; 3]) -> Result<Vector3<NonNaN<f32>>, &'static str> {
    Ok(Vector3::new(
        NonNaN::<f32>::new(v[0]).ok().ok_or("NaN in vertex")?,
        NonNaN::<f32>::new(v[1]).ok().ok_or("NaN in vertex")?,
        NonNaN::<f32>::new(v[2]).ok().ok_or("NaN in vertex")?,
    ))
}

trait Hodge {
    fn hodge_star(&self, metric: &Metric) -> Self;
}

trait ExternalDerivative {
    fn d(&mut self) -> Self;
}
