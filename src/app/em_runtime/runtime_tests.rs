use super::EmRuntime;
use crate::app::coords_sys::CoordsSys;
use crate::app::grid::Grid;
use crate::app::ui::{EmMode, EmUiState};
use crate::maths::Point;
use mathhook_core::Parser;
use std::f64::consts::PI;

fn identity_grid() -> Grid {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    Grid::new(CoordsSys::new(parse("x"), parse("y"), parse("z")))
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
