use std::collections::HashMap;
use symbolica::{atom::AtomCore, parse, symbol, LicenseManager};
use symbolica::atom::Atom;
use symbolica::numerical_integration::{ContinuousGrid, DiscreteGrid, Grid, MonteCarloRng, Sample};

#[test]
fn tests_symbolica() {
    LicenseManager::set_license_key("f08142ee#6ac7c2a5#db2f2a13-d714-5e08-bfa7-4da323f957ea");

    let input = parse!("x^2 + 2*x + 1");
    let d_input = input.derivative(symbol!("x"));
    println!("{}", d_input);

    let f = parse!("cos(x)+sin(2x)");

    // int de 0 à 1 to -2 à 2

    let func = |x: f64| -> f64 {
        let mut const_map = HashMap::default();
        const_map.insert(Atom::var(symbol!("x")), x * 4. - 4.);
        f.evaluate(|r| r.to_f64(), &const_map, &HashMap::default()).unwrap()
    };

    // let mut grid = DiscreteGrid::new(
    //     vec![
    //         Some(Grid::Continuous(ContinuousGrid::new(1, 128, 2000, None, false)))
    //     ],
    //     0.01,
    //     false,
    // );

    let mut grid = ContinuousGrid::new(1, 128, 2000, None, false);

    let mut rng = MonteCarloRng::new(0,0);
    let mut sample = Sample::new();
    for it in 1..30 {
        for _ in 0..1000 {
            grid.sample(&mut rng, &mut sample);
            // if let Sample::Discrete(_weight, i, cont_sample) = &sample {
            //     if let Sample::Continuous(_cont_weight, xs ) = cont_sample.as_ref().unwrap().as_ref() {
            //         grid.add_training_sample(&sample, func(xs[0])).unwrap();
            //     }
            // }
            if let Sample::Continuous(_cont_weight, xs ) = &sample {
                grid.add_training_sample(&sample, func(xs[0])).unwrap();
            }
        }
        grid.update(1.5);
        println!("it: {}, integral: {}, error {}, chi_sq {}", it, grid.accumulator.avg, grid.accumulator.err, grid.accumulator.chi_sq);
    }

}