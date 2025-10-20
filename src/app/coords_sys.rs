use std::collections::HashMap;
use exmex::Express;
use nalgebra::Vector3;
use symbolica::atom::{Atom, AtomCore};
use symbolica::symbol;
use crate::maths::{integrate1d, Expr, COORD};

pub struct CoordsSys {
    x_eq: Atom,
    y_eq: Atom,
    z_eq: Atom,
    x_curvature: Atom,
    y_curvature: Atom,
    z_curvature: Atom,
}

impl CoordsSys {
    pub fn new(x_eq: Atom, y_eq: Atom, z_eq: Atom) -> Self {
        let (x_curvature, y_curvature, z_curvature) = Self::calculate_curvature(&x_eq, &y_eq, &z_eq);
        Self {x_eq, y_eq, z_eq, x_curvature, y_curvature, z_curvature }
    }

    fn calculate_curvature(x_eq: &Atom, y_eq: &Atom, z_eq: &Atom) -> (Atom, Atom, Atom){
        let mut curvature = Vec::new();
        for x_i in COORD {
            let var = symbol!(x_i);
            let ddx_1 = x_eq.derivative(var).derivative(var).npow(2.);
            let ddx_2 = y_eq.derivative(var).derivative(var).npow(2.);
            let ddx_3 = z_eq.derivative(var).derivative(var).npow(2.);
            let ddx = (ddx_1 + ddx_2 + ddx_3).sqrt();
            println!("{}: {}", x_i, ddx); // TODO: when dd_xi is null, ddx becomes a sqrt of any variable which is understand as the sqrt func
            curvature.push(ddx);
        }
        let [a, b, c] = curvature.try_into().expect("COORD must have 3 elements");
        (a, b, c)
    }

    pub fn get_curvature(&self, point: Vector3<f64>, len: f64) -> (f64, f64, f64) {
        let (x, y, z) = (point.x, point.y, point.z);
        let cx = integrate1d(&self.x_curvature, (x - len, x + len));
        let cy = integrate1d(&self.y_curvature, (y - len, y + len));
        let cz = integrate1d(&self.z_curvature, (z - len, z + len));
        (cx, cy, cz)
    }

    pub fn eval(&self, x_tild: f64, y_tild: f64, z_tild: f64) -> (f64, f64, f64) {
        let X = symbol!("x");
        let Y = symbol!("y");
        let Z = symbol!("z");
        let mut const_map = HashMap::default();
        const_map.insert(Atom::var(X), x_tild);
        const_map.insert(Atom::var(Y), y_tild);
        const_map.insert(Atom::var(Z), z_tild);
        let x = self.x_eq.evaluate(|r| r.to_f64(), &const_map, &HashMap::default()).expect("x_eq must contains all 3 variables");
        let y = self.y_eq.evaluate(|r| r.to_f64(), &const_map, &HashMap::default()).expect("y_eq must contains all 3 variables");
        let z = self.z_eq.evaluate(|r| r.to_f64(), &const_map, &HashMap::default()).expect("z_eq must contains all 3 variables");
        (x, y, z)
    }
}