// tests/metric.rs
use mathhook_core::Parser;
use nalgebra::Matrix3;
use render_engine::maths::space::{Metric, Space};
use render_engine::maths::{num, Expr};
use std::collections::HashMap;

fn eval_expr_at_xyz(expr: &Expr, x: f64, y: f64, z: f64) -> f64 {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), num(x));
    vars.insert("y".to_string(), num(y));
    vars.insert("z".to_string(), num(z));
    expr.substitute_and_simplify(&vars)
        .evaluate_to_f64()
        .unwrap()
}

fn eval_metric_at_xyz(m: &Metric, x: f64, y: f64, z: f64) -> Matrix3<f64> {
    Matrix3::new(
        eval_expr_at_xyz(&m.get_element(0, 0), x, y, z),
        eval_expr_at_xyz(&m.get_element(0, 1), x, y, z),
        eval_expr_at_xyz(&m.get_element(0, 2), x, y, z),
        eval_expr_at_xyz(&m.get_element(1, 0), x, y, z),
        eval_expr_at_xyz(&m.get_element(1, 1), x, y, z),
        eval_expr_at_xyz(&m.get_element(1, 2), x, y, z),
        eval_expr_at_xyz(&m.get_element(2, 0), x, y, z),
        eval_expr_at_xyz(&m.get_element(2, 1), x, y, z),
        eval_expr_at_xyz(&m.get_element(2, 2), x, y, z),
    )
}

#[test]
fn metric_algebra_matches_known_linear_transform() {
    // Coordinate transform:
    // X = x + 2y
    // Y = 3y + z
    // Z = 4z
    //
    // In Euclidean space, metric in (x,y,z) is:
    // g = J^T J where J = d(X,Y,Z)/d(x,y,z)
    //
    // J = [1 2 0
    //      0 3 +1
    //      0 0 4]
    //
    // g = [1, 4 0,
    //      4, 13, 6,
    //      0, 6, 17]
    let x_eq = Parser::default().parse("x + 2y").unwrap();
    let y_eq = Parser::default().parse("3y + z").unwrap();
    let z_eq = Parser::default().parse("4z").unwrap();

    let space = Space::new(x_eq, y_eq, z_eq);
    // Evaluate at an arbitrary point (should be constant anyway for linear transform).
    println!("Metric: {:?}", space.get_metric());
    let g_num = eval_metric_at_xyz(space.get_metric(), 1.23, -0.7, 2.0);
    println!("g_num = {:?}", g_num);
    let expected = Matrix3::new(1.0, 4.0, 0.0, 4.0, 13.0, 6.0, 0.0, 6.0, 17.0);

    let tol = 1e-10;
    for i in 0..3 {
        for j in 0..3 {
            assert!(
                (g_num[(i, j)] - expected[(i, j)]).abs() < tol,
                "g[{},{}] = {}, expected {}",
                i,
                j,
                g_num[(i, j)],
                expected[(i, j)]
            );
        }
    }
}
