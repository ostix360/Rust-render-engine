#![allow(unused)]

use std::collections::HashMap;
use exmex::{FlatEx, FloatOpsFactory};
use nalgebra::Vector3;
use symbolica::atom::{Atom, AtomCore};
use symbolica::evaluate::{FunctionMap, OptimizationSettings};
use symbolica::numerical_integration::{ContinuousGrid, MonteCarloRng, Sample};
use symbolica::{parse, symbol};
use typed_floats::NonNaN;

mod differential;
mod space;

pub type Expr = FlatEx<f32, FloatOpsFactory<f32>>;

pub const COORD: [&str; 3] = ["x", "y", "z"];

pub fn integrate1d(f: &Atom, interval: (f64, f64)) -> f64 {
    let (a, b) = interval;
    let len = b - a;
    let fn_map = FunctionMap::new();
    let params = parse!("x");
    let optimization_settings = OptimizationSettings::default();
    let mut evaluator = f.evaluator(
        &fn_map, &[params], optimization_settings
    ).unwrap().map_coeff(&|x| {
        x.to_real().unwrap().to_f64()
    });
    let func = |x: f64| -> f64 {
        let mut const_map = HashMap::default();
        const_map.insert(Atom::var(symbol!("x")), x);
        f.evaluate(|r| r.to_f64(), &const_map, &HashMap::default()).unwrap()
    };

    // let mut grid = DiscreteGrid::new(
    //     vec![
    //         Some(Grid::Continuous(ContinuousGrid::new(1, 128, 2000, None, false)))
    //     ],
    //     0.01,
    //     false,
    // );

    let mut grid = ContinuousGrid::new(1, 20, 2000, None, false);

    let mut rng = MonteCarloRng::new(0,0,);
    let mut sample = Sample::new();
    for it in 1..10 {
        for _ in 0..500 {
            grid.sample(&mut rng, &mut sample);
            // if let Sample::Discrete(_weight, i, cont_sample) = &sample {
            //     if let Sample::Continuous(_cont_weight, xs ) = cont_sample.as_ref().unwrap().as_ref() {
            //         grid.add_training_sample(&sample, func(xs[0])).unwrap();
            //     }
            // }
            if let Sample::Continuous(_cont_weight, xs ) = &sample{
                grid.add_training_sample(&sample, evaluator.evaluate_single(&[xs[0] * len + a])).unwrap();
            }
        }
        grid.update(1.5);
    };
    grid.accumulator.avg
}

pub fn to_nn_vec(v: [f32; 3]) -> Result<Vector3<NonNaN<f32>>, &'static str> {
    Ok(Vector3::new(
        NonNaN::<f32>::new(v[0]).ok().ok_or("NaN in vertex")?,
        NonNaN::<f32>::new(v[1]).ok().ok_or("NaN in vertex")?,
        NonNaN::<f32>::new(v[2]).ok().ok_or("NaN in vertex")?,
    ))
}

trait Hodge {
    fn hodge_star(&self) -> Self;
}

trait ExternalDerivative {
    fn d(&self) -> Self;
}

