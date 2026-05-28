use super::EmRuntime;
use crate::app::coords_sys::CoordsSys;
use crate::app::grid::{Grid, GridConfig};
use crate::app::ui::{EmMode, EmUiState};
use crate::maths::Point;
use mathhook_core::Parser;
use std::f64::consts::PI;

fn identity_grid() -> Grid {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    Grid::new(CoordsSys::new(parse("x"), parse("y"), parse("z")))
}

fn scaled_cartesian_grid() -> Grid {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    Grid::new(CoordsSys::new(parse("2*x"), parse("3*y"), parse("4*z")))
}

fn spherical_grid() -> Grid {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    Grid::new(CoordsSys::new(
        parse("x*cos(y) * sin(z)"),
        parse("x*sin(y) * sin(z)"),
        parse("x * cos(z)"),
    ))
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
}

fn assert_close_tol(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{actual} != {expected}"
    );
}

fn assert_near_zero(actual: f64) {
    assert_close(actual, 0.0);
}

#[test]
fn plane_wave_shortcut_requires_orthonormal_cartesian_geometry() {
    assert!(identity_grid()
        .get_coords()
        .sample_geometry()
        .is_orthonormal_cartesian());
    assert!(!scaled_cartesian_grid()
        .get_coords()
        .sample_geometry()
        .is_orthonormal_cartesian());
    assert!(!spherical_grid()
        .get_coords()
        .sample_geometry()
        .is_orthonormal_cartesian());
}

#[test]
fn potential_mode_computes_expected_electric_field() {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    let mut state = EmUiState::default();
    state.mode = EmMode::Potentials;
    state.phi.eq = parse("x * t");
    state.vector_potential.x.eq = parse("0");
    state.vector_potential.y.eq = parse("y * t");
    state.vector_potential.z.eq = parse("0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let value = runtime.electric_at(
        Point {
            x: 3.0,
            y: 4.0,
            z: 0.0,
        },
        2.0,
    );

    assert_close(value.x, -2.0);
    assert_close(value.y, -4.0);
    assert_close(value.z, 0.0);
}

#[test]
fn potential_mode_computes_expected_magnetic_field() {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    let mut state = EmUiState::default();
    state.mode = EmMode::Potentials;
    state.vector_potential.x.eq = parse("0");
    state.vector_potential.y.eq = parse("x * z");
    state.vector_potential.z.eq = parse("0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let value = runtime.magnetic_at(
        Point {
            x: 2.0,
            y: 0.0,
            z: 5.0,
        },
        0.0,
    );

    assert_close(value.x, -2.0);
    assert_close(value.y, 0.0);
    assert_close(value.z, 5.0);
}

#[test]
fn potential_mode_plane_wave_magnetic_field_oscillates_without_rotation() {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    let mut state = EmUiState::default();
    state.mode = EmMode::Potentials;
    state.phi.eq = parse("0");
    state.vector_potential.x.eq = parse("0");
    state.vector_potential.y.eq = parse("sin(z - t)");
    state.vector_potential.z.eq = parse("0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let point = Point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let initial = runtime.magnetic_at(point, 0.0);
    let half_turn = runtime.magnetic_at(point, PI);

    assert_close(initial.x, -1.0);
    assert_near_zero(initial.y);
    assert_near_zero(initial.z);
    assert_close(half_turn.x, 1.0);
    assert_near_zero(half_turn.y);
    assert_near_zero(half_turn.z);
}

#[test]
fn electric_source_uses_ampere_time_derivative_for_magnetic_field() {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    let mut state = EmUiState::default();
    state.mode = EmMode::Electric;
    state.electric_field.x.eq = parse("0");
    state.electric_field.y.eq = parse("t");
    state.electric_field.z.eq = parse("0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let point = Point {
        x: 2.0,
        y: 3.0,
        z: 4.0,
    };
    let unit_c_value = runtime.magnetic_at(point, 0.0).norm();

    state.light_speed = 2.0;
    let slower_runtime = EmRuntime::from_ui(&state, &identity_grid());
    let slower_value = slower_runtime.magnetic_at(point, 0.0).norm();

    assert!(unit_c_value > 1.0e-6);
    assert_close_tol(slower_value, unit_c_value * 0.25, 1.0e-6);
}

#[test]
fn electric_source_inverse_curl_stays_bounded_in_spherical_geometry() {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    let mut state = EmUiState::default();
    state.mode = EmMode::Electric;
    state.electric_field.x.eq = parse("0");
    state.electric_field.y.eq = parse("1/x * cos(x - t)");
    state.electric_field.z.eq = parse("0");

    let runtime = EmRuntime::from_ui_with_config(
        &state,
        &spherical_grid(),
        GridConfig::new(0.0, 6.0, 5.0, 0.0, 6.28, 12.0, 0.0, 3.14, 8.0),
    );
    let value = runtime.magnetic_at(
        Point {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        0.0,
    );

    assert!(value.x.is_finite() && value.y.is_finite() && value.z.is_finite());
    assert!(
        value.norm() < 25.0,
        "unexpected spherical B spike: {value:?}"
    );

    for point in [
        Point {
            x: 1.0,
            y: 0.0,
            z: 1.57,
        },
        Point {
            x: 1.0,
            y: 3.14,
            z: 1.57,
        },
        Point {
            x: 2.0,
            y: 1.0,
            z: 1.0,
        },
    ] {
        let value = runtime.magnetic_at(point, 0.0);
        assert!(
            value.norm() < 25.0,
            "unexpected spherical B spike at ({}, {}, {}): {value:?}",
            point.x,
            point.y,
            point.z
        );
    }
}

#[test]
fn electric_source_plane_wave_magnetic_field_oscillates_without_rotation() {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    let mut state = EmUiState::default();
    state.mode = EmMode::Electric;
    state.electric_field.x.eq = parse("0");
    state.electric_field.y.eq = parse("cos(z - t)");
    state.electric_field.z.eq = parse("0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let point = Point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let initial = runtime.magnetic_at(point, 0.0);
    let half_turn = runtime.magnetic_at(point, PI);

    assert_close(initial.x, -1.0);
    assert_near_zero(initial.y);
    assert_near_zero(initial.z);
    assert_close(half_turn.x, 1.0);
    assert_near_zero(half_turn.y);
    assert_near_zero(half_turn.z);
}

#[test]
fn electric_source_plane_wave_falls_back_for_scaled_cartesian_geometry() {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    let mut state = EmUiState::default();
    state.mode = EmMode::Electric;
    state.electric_field.x.eq = parse("0");
    state.electric_field.y.eq = parse("cos(z - t)");
    state.electric_field.z.eq = parse("0");

    let runtime = EmRuntime::from_ui(&state, &scaled_cartesian_grid());
    let point = Point {
        x: 3.0,
        y: 3.0,
        z: 3.0,
    };
    let value = runtime.magnetic_at(point, 0.5);
    let shortcut_x = -(point.z - 0.5).cos();

    assert!(
        (value.x - shortcut_x).abs() > 1.0e-3 || value.y.abs() > 1.0e-3 || value.z.abs() > 1.0e-3,
        "scaled Cartesian direct source should use inverse-curl fallback, not shortcut B: {value:?}"
    );
}

#[test]
fn magnetic_source_uses_faraday_time_derivative_for_electric_field() {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    let mut state = EmUiState::default();
    state.mode = EmMode::Magnetic;
    state.magnetic_field.x.eq = parse("t");
    state.magnetic_field.y.eq = parse("0");
    state.magnetic_field.z.eq = parse("0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let value = runtime.electric_at(
        Point {
            x: 2.0,
            y: 3.0,
            z: 4.0,
        },
        0.0,
    );

    assert!(value.norm() > 1.0e-6);
}
