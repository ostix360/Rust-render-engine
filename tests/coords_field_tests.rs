use mathhook_core::Parser;
use nalgebra::{vector, Vector3};
use render_engine::app::coords_sys::CoordsSys;
use render_engine::maths::differential::Form;
use render_engine::maths::field::VectorField;
use render_engine::maths::space::Space;
use render_engine::maths::expr_to_fastexpr3d;
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

#[test]
fn vector_field_dual_at_matches_dual_component_expressions() {
    let space = Space::new(parse("x"), parse("y"), parse("z"));
    let field = VectorField::new(
        Form::new(vec![parse("x"), parse("y + 2"), parse("3*z")], 1),
        &space,
    );

    let result = field.dual_at(Point {
        x: 1.5,
        y: -2.0,
        z: 0.25,
    });

    assert_close(result.x, 1.5, "dual_at x component");
    assert_close(result.y, 0.0, "dual_at y component");
    assert_close(result.z, 0.75, "dual_at z component");
}

#[test]
fn vector_field_linearized_at_keeps_constant_field_constant() {
    let space = Space::new(parse("x"), parse("y"), parse("z"));
    let field = VectorField::from_otn(
        Form::new(vec![parse("1"), parse("1"), parse("1")], 1),
        &space,
    );

    let result = field.linearized_at(
        Point {
            x: 2.0,
            y: 3.0,
            z: 4.0,
        },
        Point {
            x: 0.25,
            y: -0.5,
            z: 1.5,
        },
    );

    assert_close(result.x, 1.0, "linearized constant x");
    assert_close(result.y, 1.0, "linearized constant y");
    assert_close(result.z, 1.0, "linearized constant z");
}

#[test]
fn vector_field_linearized_at_matches_affine_field_exactly() {
    let space = Space::new(parse("x"), parse("y"), parse("z"));
    let field = VectorField::from_otn(
        Form::new(vec![parse("x + 2*y"), parse("3 - y"), parse("z - 4*x")], 1),
        &space,
    );
    let anchor = Point {
        x: 1.0,
        y: -2.0,
        z: 0.5,
    };
    let delta = Point {
        x: 0.25,
        y: 0.5,
        z: -0.75,
    };

    let linearized = field.linearized_at(anchor, delta);
    let exact = field.at(Point {
        x: anchor.x + delta.x,
        y: anchor.y + delta.y,
        z: anchor.z + delta.z,
    });

    assert_close(linearized.x, exact.x, "linearized affine x");
    assert_close(linearized.y, exact.y, "linearized affine y");
    assert_close(linearized.z, exact.z, "linearized affine z");
}

#[test]
fn vector_field_dual_at_preserves_all_scaled_components() {
    let space = Space::new(parse("2*x"), parse("3*y"), parse("4*z"));
    let field = VectorField::from_otn(
        Form::new(vec![parse("1"), parse("2"), parse("3")], 1),
        &space,
    );

    let result = field.dual_at(Point {
        x: 2.0,
        y: 0.0,
        z: 1.0,
    });

    assert_close(result.x, 2.0, "scaled dual x component");
    assert_close(result.y, 6.0, "scaled dual y component");
    assert_close(result.z, 12.0, "scaled dual z component");
}

#[test]
fn gradient_from_scalar_matches_cartesian_derivatives() {
    let space = Space::new(parse("x"), parse("y"), parse("z"));
    let field = VectorField::gradient_from_scalar(parse("x*y + z*z"), &space);

    let result = field.at(Point {
        x: 2.0,
        y: -3.0,
        z: 0.5,
    });

    assert_close(result.x, -3.0, "gradient x component");
    assert_close(result.y, 2.0, "gradient y component");
    assert_close(result.z, 1.0, "gradient z component");
}

#[test]
fn curl_from_otn_matches_cartesian_curl() {
    let space = Space::new(parse("x"), parse("y"), parse("z"));
    let field = VectorField::curl_from_otn(
        Form::new(vec![parse("-y"), parse("x"), parse("0")], 1),
        &space,
    );

    let result = field.at(Point {
        x: 1.0,
        y: 2.0,
        z: -3.0,
    });

    assert_close(result.x, 0.0, "curl x component");
    assert_close(result.y, 0.0, "curl y component");
    assert_close(result.z, 2.0, "curl z component");
}

#[test]
fn hodge_star_otn_3d_maps_basis_forms() {
    let one_form = Form::new(vec![parse("1"), parse("2"), parse("3")], 1).hodge_star_otn_3d();

    assert_eq!(one_form.n_forms(), 2);
    assert_close(
        expr_to_fastexpr3d(one_form.get_expr(0).clone())(0.0, 0.0, 0.0),
        3.0,
        "star one-form xy component",
    );
    assert_close(
        expr_to_fastexpr3d(one_form.get_expr(1).clone())(0.0, 0.0, 0.0),
        1.0,
        "star one-form yz component",
    );
    assert_close(
        expr_to_fastexpr3d(one_form.get_expr(2).clone())(0.0, 0.0, 0.0),
        2.0,
        "star one-form zx component",
    );

    let two_form = Form::new(vec![parse("5"), parse("7"), parse("11")], 2).hodge_star_otn_3d();

    assert_eq!(two_form.n_forms(), 1);
    assert_close(
        expr_to_fastexpr3d(two_form.get_expr(0).clone())(0.0, 0.0, 0.0),
        7.0,
        "star two-form x component",
    );
    assert_close(
        expr_to_fastexpr3d(two_form.get_expr(1).clone())(0.0, 0.0, 0.0),
        11.0,
        "star two-form y component",
    );
    assert_close(
        expr_to_fastexpr3d(two_form.get_expr(2).clone())(0.0, 0.0, 0.0),
        5.0,
        "star two-form z component",
    );
}

#[test]
fn two_form_to_otn_base_handles_scaled_orthogonal_metric() {
    let space = Space::new(parse("2*x"), parse("3*y"), parse("4*z"));
    let transformed = Form::new(vec![parse("1"), parse("0"), parse("0")], 2).to_otn_base(&space);
    let eval = |index| expr_to_fastexpr3d(transformed.get_expr(index).clone())(0.0, 0.0, 0.0);

    assert_close(
        eval(0),
        1.0 / 6.0,
        "transformed xy component",
    );
    assert_close(
        eval(1),
        0.0,
        "transformed yz component",
    );
    assert_close(
        eval(2),
        0.0,
        "transformed zx component",
    );
}
