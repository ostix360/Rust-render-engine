use mathhook_core::Parser;
use nalgebra::{vector, Vector3};
use render_engine::app::coords_sys::CoordsSys;
use render_engine::maths::differential::Form;
use render_engine::maths::field::VectorField;
use render_engine::maths::space::Space;
use render_engine::maths::Point;

const EPS: f64 = 1.0e-6;

fn parse(expr: &str) -> render_engine::maths::Expr {
    Parser::default().parse(expr).unwrap()
}

fn assert_close(actual: f64, expected: f64, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= EPS,
        "{}: expected {:.8}, got {:.8} (delta {:.8})",
        context,
        expected,
        actual,
        delta
    );
}

fn assert_vec3_close(actual: Vector3<f64>, expected: Vector3<f64>, context: &str) {
    assert_close(actual.x, expected.x, &format!("{context} x"));
    assert_close(actual.y, expected.y, &format!("{context} y"));
    assert_close(actual.z, expected.z, &format!("{context} z"));
}

#[test]
fn coords_sys_eval_position_matches_identity_coordinates() {
    let coords = CoordsSys::new(parse("x"), parse("y"), parse("z"));
    let point = vector![1.25, -2.5, 0.75];

    let result = coords.eval_position(point);

    assert_vec3_close(result, point, "identity position");
}

#[test]
fn coords_sys_eval_position_matches_spherical_coordinates() {
    let coords = CoordsSys::new(
        parse("x*cos(y) * sin(z)"),
        parse("x*sin(y) * sin(z)"),
        parse("x * cos(z)"),
    );
    let point = vector![2.0, 0.0, std::f64::consts::FRAC_PI_2];

    let result = coords.eval_position(point);

    assert_vec3_close(result, vector![2.0, 0.0, 0.0], "spherical position");
}

#[test]
fn eval_otn_vector_preserves_components_in_identity_space() {
    let coords = CoordsSys::new(parse("x"), parse("y"), parse("z"));
    let point = vector![1.0, 2.0, 3.0];
    let vector = vector![0.5, -1.5, 2.25];

    let result = coords.eval_otn_vector(point, vector);

    assert_vec3_close(result, vector, "identity otn vector");
}

#[test]
fn eval_otn_vector_matches_expected_spherical_basis_at_x_axis() {
    let coords = CoordsSys::new(
        parse("x*cos(y) * sin(z)"),
        parse("x*sin(y) * sin(z)"),
        parse("x * cos(z)"),
    );
    let point = vector![2.0, 0.0, std::f64::consts::FRAC_PI_2];
    let vector = vector![1.0, 2.0, 3.0];

    let result = coords.eval_otn_vector(point, vector);

    assert_vec3_close(result, vector![1.0, 2.0, -3.0], "spherical basis on x axis");
}

#[test]
fn eval_otn_vector_matches_expected_spherical_basis_at_y_axis() {
    let coords = CoordsSys::new(
        parse("x*cos(y) * sin(z)"),
        parse("x*sin(y) * sin(z)"),
        parse("x * cos(z)"),
    );
    let point = vector![
        2.0,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::FRAC_PI_2
    ];
    let vector = vector![1.0, 2.0, 3.0];

    let result = coords.eval_otn_vector(point, vector);

    assert_vec3_close(
        result,
        vector![-2.0, 1.0, -3.0],
        "spherical basis on y axis",
    );
}

#[test]
fn vector_field_from_otn_evaluates_component_expressions() {
    let space = Space::new(parse("x"), parse("y"), parse("z"));
    let field = VectorField::from_otn(
        Form::new(vec![parse("x + 1"), parse("2*y"), parse("z - 3")], 1),
        &space,
    );

    let result = field.at(Point {
        x: 2.0,
        y: -1.5,
        z: 4.0,
    });

    assert_close(result.x, 3.0, "field x component");
    assert_close(result.y, -3.0, "field y component");
    assert_close(result.z, 1.0, "field z component");
}

#[test]
fn vector_field_new_preserves_dual_components_in_identity_space() {
    let space = Space::new(parse("x"), parse("y"), parse("z"));
    let field = VectorField::new(
        Form::new(vec![parse("x"), parse("y + 2"), parse("3*z")], 1),
        &space,
    );

    let dual = field.get_dual();
    let point = Point {
        x: 1.5,
        y: -2.0,
        z: 0.25,
    };
    let vars = std::collections::HashMap::from([
        ("x".to_string(), render_engine::maths::num(point.x)),
        ("y".to_string(), render_engine::maths::num(point.y)),
        ("z".to_string(), render_engine::maths::num(point.z)),
    ]);

    assert_close(
        dual.get_expr(0)
            .substitute(&vars)
            .evaluate_to_f64()
            .unwrap(),
        1.5,
        "dual x component",
    );
    assert_close(
        dual.get_expr(1)
            .substitute(&vars)
            .evaluate_to_f64()
            .unwrap(),
        0.0,
        "dual y component",
    );
    assert_close(
        dual.get_expr(2)
            .substitute(&vars)
            .evaluate_to_f64()
            .unwrap(),
        0.75,
        "dual z component",
    );
}
