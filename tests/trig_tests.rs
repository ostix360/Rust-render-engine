use std::f64::consts::FRAC_PI_2;
use nalgebra::SimdComplexField;

#[test]
fn main() {
    let a: f64 = FRAC_PI_2;
    let cos_a = a.cos();
    let sin_a = a.sin();
    let (b,c) = a.simd_sin_cos();
    println!("cos_a: {}, sin_a: {}", cos_a, sin_a);
    println!("b: {}, c: {}", b, c);
}