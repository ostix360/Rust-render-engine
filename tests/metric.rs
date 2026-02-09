// tests/metric.rs
use exmex::{parse, Express};
use nalgebra::Matrix3;
use render_engine::maths::space::{Metric, Space};
use render_engine::maths::Expr;

fn eval_expr_at_xyz(expr: &Expr, x: f64, y: f64, z: f64) -> f64 {
    let args = expr
        .var_names()
        .iter()
        .map(|name| match name.as_str() {
            "x" => x,
            "y" => y,
            "z" => z,
            unknown => panic!("unexpected variable in metric expression: {unknown}"),
        })
        .collect::<Vec<_>>();
    expr.eval_relaxed(&args).unwrap()
}

fn eval_metric_at_xyz(m: &Metric, x: f64, y: f64, z: f64) -> Matrix3<f64> {
    Matrix3::new(
        eval_expr_at_xyz(&m[0][0], x, y, z),
        eval_expr_at_xyz(&m[0][1], x, y, z),
        eval_expr_at_xyz(&m[0][2], x, y, z),
        eval_expr_at_xyz(&m[1][0], x, y, z),
        eval_expr_at_xyz(&m[1][1], x, y, z),
        eval_expr_at_xyz(&m[1][2], x, y, z),
        eval_expr_at_xyz(&m[2][0], x, y, z),
        eval_expr_at_xyz(&m[2][1], x, y, z),
        eval_expr_at_xyz(&m[2][2], x, y, z),
    )
}



#[test]
fn metric_algebra_matches_known_linear_transform() {
    // Coordinate transform:
    // X = x + 2y
    // Y = 3y - z
    // Z = 4z
    //
    // In Euclidean space, metric in (x,y,z) is:
    // g = J^T J where J = d(X,Y,Z)/d(x,y,z)
    //
    // J = [1 2 0
    //      0 3 -1
    //      0 0 4]
    //
    // g = [1 2 0;
    //      2 13 -3;
    //      0 -3 17]
    let x_eq = parse("x").unwrap();
    let y_eq = parse("y").unwrap();
    let z_eq = parse("z").unwrap();

    let space = Space::new(x_eq, y_eq, z_eq);
    // Evaluate at an arbitrary point (should be constant anyway for linear transform).

    let g_num = eval_metric_at_xyz(space.get_metric(), 1.23, -0.7, 2.0);

    let expected = Matrix3::new(1.0, 0., 0.0, 0.0, 1.0, 0., 0.0, -0.0, 1.0);

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
