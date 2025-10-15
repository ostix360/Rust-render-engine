#![allow(unused)]

use std::collections::HashMap;
use exmex::{FlatEx, FloatOpsFactory};
use nalgebra::Vector3;
use symbolica::atom::{Atom, AtomCore};
use symbolica::numerical_integration::{ContinuousGrid, MonteCarloRng, Sample};
use symbolica::symbol;
use typed_floats::NonNaN;

mod differential;
mod space;

pub type Expr = FlatEx<f32, FloatOpsFactory<f32>>;

pub const COORD: [&str; 3] = ["x", "y", "z"];

pub fn integrate1d(f: &Atom, interval: (f64, f64)) -> f64 {
    let (a, b) = interval;
    let len = b - a;
    
    let func = |x: f64| -> f64 {
        let mut const_map = HashMap::default();
        const_map.insert(Atom::var(symbol!("x")), x * len + a);
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
    for it in 1..30 {
        for _ in 0..1000 {
            grid.sample(&mut rng, &mut sample);
            // if let Sample::Discrete(_weight, i, cont_sample) = &sample {
            //     if let Sample::Continuous(_cont_weight, xs ) = cont_sample.as_ref().unwrap().as_ref() {
            //         grid.add_training_sample(&sample, func(xs[0])).unwrap();
            //     }
            // }
            if let Sample::Continuous(_cont_weight, xs ) = &sample{
                grid.add_training_sample(&sample, func(xs[0])).unwrap();
            }
        }
        grid.update(1.5);
        println!("it: {}, integral: {}, error {}, chi_sq {}", it, grid.accumulator.avg, grid.accumulator.err, grid.accumulator.chi_sq);
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

