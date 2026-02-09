#![allow(unused)]

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use egui::TextBuffer;
use exmex::{Calculate, Differentiate, Express, FlatEx, FloatOpsFactory};
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use nalgebra::Vector3;
use symbolica::atom::{Atom, AtomCore};
use symbolica::evaluate::{Expression, FunctionMap, OptimizationSettings};
use symbolica::numerical_integration::{ContinuousGrid, MonteCarloRng, Sample};
use symbolica::{parse, symbol};
use symbolica::printer::PrintOptions;
use typed_floats::NonNaN;
use crate::maths::space::Metric;

pub mod differential;
pub mod space;

pub type Expr = FlatEx<f64, FloatOpsFactory<f64>>;
pub type FastExpr1d = Arc<dyn Fn(f64) -> f64 + Sync>;

pub type FastExpr2dto1d = Arc<dyn Fn(f64, f64) -> FastExpr1d>;
pub type FastExpr3d = Arc<dyn Fn(f64, f64, f64) -> f64 + Sync>;

pub const COORD: [&str; 3] = ["x", "y", "z"];

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
    expr.compile();

    let eval = move |x:f64| -> f64{
        expr.eval_relaxed(&[x]).unwrap()
    };
    Arc::new(eval)
}

#[inline]
pub fn expr_to_fastexpr2dto1d(mut expr: Expr, var_name: String) -> FastExpr2dto1d {
    expr.compile();
    let expr = Arc::new(expr);
    #[inline]
    fn select_arg(name: &str, target: &str, value: f64, first: f64, second: f64) -> f64 {
        match (name, target) {
            ("x", "x") => value,
            ("y", "x") => first,
            ("z", "x") => second,
            ("x", "y") => first,
            ("y", "y") => value,
            ("z", "y") => second,
            ("x", "z") => first,
            ("y", "z") => second,
            ("z", "z") => value,
            _ => panic!("Unknown variable {} while fixing {}", name, target),
        }
    }
    let func = move |x_: f64, y_: f64| -> FastExpr1d {
        let expr = expr.clone();
        let target_name = var_name.clone();

        let eval = move |value: f64| -> f64 {
            let names = expr.var_names();
            let args = names
                .iter()
                .map(|name| select_arg(name, target_name.as_str(), value, x_, y_))
                .collect::<Vec<_>>();
            expr.eval_relaxed(&args).unwrap()
        };
        Arc::new(eval)
    };
    Arc::new(func)
}

pub fn expr_to_fastexpr3d(mut expr: Expr) -> FastExpr3d {
    expr.compile();
    let eval = move |x:f64, y:f64, z:f64| -> f64{
        let vars = expr.var_names()
            .iter().map(|name| match name.as_str() {
                "x" => x,
                "y" => y,
                "z" => z,
                _ => panic!("Unknown variable name {}", name),
        }).collect::<Vec<_>>();
        expr.eval_relaxed(&vars).unwrap()
    };
    Arc::new(eval)
}

pub fn derivate(expr: Expr, variable_name: &String) -> Expr {
    if variable_name == "x" && expr.var_names().contains(variable_name) {
        let mut out = expr.clone().partial(0).unwrap();
        out
    }else if variable_name == "y" && expr.var_names().contains(variable_name) {
        if !expr.var_names().contains(&"x".to_string()) {
            expr.partial(0).unwrap()
        } else {
            expr.partial(1).unwrap()
        }
    }else if variable_name == "z" && expr.var_names().contains(variable_name) {
        if !expr.var_names().contains(&"x".to_string()) && !expr.var_names().contains(&"y".to_string()) {
            expr.partial(0).unwrap()
        } else if !expr.var_names().contains(&"x".to_string()) || !expr.var_names().contains(&"y".to_string()) {
            expr.partial(1).unwrap()
        } else {
            expr.partial(2).unwrap()
        }
    }else {
        Expr::from_num(0f64)
    }
}

pub fn to_nn_vec(v: [f32; 3]) -> Result<Vector3<NonNaN<f32>>, &'static str> {
    Ok(Vector3::new(
        NonNaN::<f32>::new(v[0]).ok().ok_or("NaN in vertex")?,
        NonNaN::<f32>::new(v[1]).ok().ok_or("NaN in vertex")?,
        NonNaN::<f32>::new(v[2]).ok().ok_or("NaN in vertex")?,
    ))
}

trait Hodge {
    fn hodge_star(&self, metric: Metric) -> Self;
}

trait ExternalDerivative {
    fn d(&mut self) -> Self;
}

