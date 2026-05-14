use mathhook_core::Parser;
use nalgebra::{vector, Vector3};
use render_engine::app::coords_sys::CoordsSys;
use render_engine::app::em_runtime::EmRuntime;
use render_engine::app::grid::Grid;
use render_engine::app::ui::{EmMode, EmUiState, EqRender, SpacialEqs};
use render_engine::maths::{Expr, Point};

const EPS: f64 = 1.0e-6;

fn parse(expr: &str) -> Expr {
    Parser::default().parse(expr).unwrap()
}

fn grid(x: &str, y: &str, z: &str) -> Grid {
    Grid::new(CoordsSys::new(parse(x), parse(y), parse(z)))
}

fn identity_grid() -> Grid {
    grid("x", "y", "z")
}

fn scaled_cartesian_grid() -> Grid {
    grid("2*x", "3*y", "4*z")
}

fn spherical_grid() -> Grid {
    grid("x*cos(y) * sin(z)", "x*sin(y) * sin(z)", "x * cos(z)")
}

fn exprs(x: &str, y: &str, z: &str) -> SpacialEqs {
    SpacialEqs {
        x: EqRender::new(parse(x), x.to_string()),
        y: EqRender::new(parse(y), y.to_string()),
        z: EqRender::new(parse(z), z.to_string()),
    }
}

fn assert_close(actual: f64, expected: f64, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= EPS,
        "{context}: expected {expected:.8}, got {actual:.8} (delta {delta:.8})"
    );
}

fn assert_close_tol(actual: f64, expected: f64, tolerance: f64, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "{context}: expected {expected:.8}, got {actual:.8} (delta {delta:.8})"
    );
}

fn assert_vector_close(actual: Vector3<f64>, expected: Vector3<f64>, context: &str) {
    assert_close(actual.x, expected.x, &format!("{context} x"));
    assert_close(actual.y, expected.y, &format!("{context} y"));
    assert_close(actual.z, expected.z, &format!("{context} z"));
}

fn finite_curl(
    field: impl Fn(Point, f64) -> Vector3<f64>,
    point: Point,
    time: f64,
    h: f64,
) -> Vector3<f64> {
    let dx = |offset: f64| Point {
        x: point.x + offset,
        ..point
    };
    let dy = |offset: f64| Point {
        y: point.y + offset,
        ..point
    };
    let dz = |offset: f64| Point {
        z: point.z + offset,
        ..point
    };

    let d_dy = (field(dy(h), time) - field(dy(-h), time)) / (2.0 * h);
    let d_dz = (field(dz(h), time) - field(dz(-h), time)) / (2.0 * h);
    let d_dx = (field(dx(h), time) - field(dx(-h), time)) / (2.0 * h);

    vector![d_dy.z - d_dz.y, d_dz.x - d_dx.z, d_dx.y - d_dy.x]
}

fn finite_divergence(
    field: impl Fn(Point, f64) -> Vector3<f64>,
    point: Point,
    time: f64,
    h: f64,
) -> f64 {
    let dx = |offset: f64| Point {
        x: point.x + offset,
        ..point
    };
    let dy = |offset: f64| Point {
        y: point.y + offset,
        ..point
    };
    let dz = |offset: f64| Point {
        z: point.z + offset,
        ..point
    };

    (field(dx(h), time).x - field(dx(-h), time).x) / (2.0 * h)
        + (field(dy(h), time).y - field(dy(-h), time).y) / (2.0 * h)
        + (field(dz(h), time).z - field(dz(-h), time).z) / (2.0 * h)
}

fn finite_partial_t(
    field: impl Fn(Point, f64) -> Vector3<f64>,
    point: Point,
    time: f64,
    h: f64,
) -> Vector3<f64> {
    (field(point, time + h) - field(point, time - h)) / (2.0 * h)
}

#[test]
fn em_runtime_potential_mode_spatial_amplitude_wave_matches_analytical_curl() {
    let mut state = EmUiState::default();
    state.mode = EmMode::Potentials;
    state.phi.eq = parse("0");
    state.vector_potential = exprs("0", "x * sin(z - t)", "0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let point = Point {
        x: 2.0,
        y: 0.0,
        z: 0.25,
    };
    let time = -0.75;
    let phase = point.z - time;

    assert_vector_close(
        runtime.electric_at(point, time),
        vector![0.0, point.x * phase.cos(), 0.0],
        "E",
    );
    assert_vector_close(
        runtime.magnetic_at(point, time),
        vector![-point.x * phase.cos(), 0.0, phase.sin()],
        "B",
    );
}

#[test]
fn em_runtime_potential_mode_standing_wave_matches_analytical_fields() {
    let mut state = EmUiState::default();
    state.mode = EmMode::Potentials;
    state.phi.eq = parse("0");
    state.vector_potential = exprs("0", "sin(z - t) + sin(z + t)", "0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let point = Point {
        x: 0.0,
        y: 0.0,
        z: 0.4,
    };
    let time = 0.6;

    assert_vector_close(
        runtime.electric_at(point, time),
        vector![0.0, (point.z - time).cos() - (point.z + time).cos(), 0.0],
        "standing E",
    );
    assert_vector_close(
        runtime.magnetic_at(point, time),
        vector![-(point.z - time).cos() - (point.z + time).cos(), 0.0, 0.0],
        "standing B",
    );
}

#[test]
fn em_runtime_potential_mode_damped_wave_matches_bounded_analytical_fields() {
    let mut state = EmUiState::default();
    state.mode = EmMode::Potentials;
    state.phi.eq = parse("0");
    state.vector_potential = exprs("0", "exp(-0.25*z) * sin(z - t)", "0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let point = Point {
        x: 0.0,
        y: 0.0,
        z: 1.5,
    };
    let time = 0.25;
    let phase = point.z - time;
    let envelope = (-0.25 * point.z).exp();

    assert_vector_close(
        runtime.electric_at(point, time),
        vector![0.0, envelope * phase.cos(), 0.0],
        "damped E",
    );
    assert_vector_close(
        runtime.magnetic_at(point, time),
        vector![envelope * (0.25 * phase.sin() - phase.cos()), 0.0, 0.0],
        "damped B",
    );
}

#[test]
fn em_runtime_potential_mode_scaled_cartesian_phi_uses_metric_gradient() {
    let mut state = EmUiState::default();
    state.mode = EmMode::Potentials;
    state.phi.eq = parse("x");
    state.vector_potential = exprs("0", "0", "0");

    let runtime = EmRuntime::from_ui(&state, &scaled_cartesian_grid());
    let value = runtime.electric_at(
        Point {
            x: 3.0,
            y: 0.0,
            z: 0.0,
        },
        0.0,
    );

    assert_vector_close(value, vector![-0.5, 0.0, 0.0], "scaled gradient");
}

#[test]
fn em_runtime_potential_mode_spherical_radial_phi_uses_metric_gradient() {
    let mut state = EmUiState::default();
    state.mode = EmMode::Potentials;
    state.phi.eq = parse("x*x");
    state.vector_potential = exprs("0", "0", "0");

    let runtime = EmRuntime::from_ui(&state, &spherical_grid());
    let value = runtime.electric_at(
        Point {
            x: 2.0,
            y: 0.7,
            z: 1.1,
        },
        0.0,
    );

    assert_vector_close(value, vector![-4.0, 0.0, 0.0], "spherical gradient");
}

#[test]
fn em_runtime_electric_source_derives_plane_wave_magnetic_field_for_each_axis() {
    let cases = [
        (exprs("0", "0", "cos(x - t)"), vector![0.0, -1.0, 0.0]),
        (exprs("cos(y - t)", "0", "0"), vector![0.0, 0.0, -1.0]),
        (exprs("0", "cos(z - t)", "0"), vector![-1.0, 0.0, 0.0]),
    ];

    for (source, expected_b) in cases {
        let mut state = EmUiState::default();
        state.mode = EmMode::Electric;
        state.electric_field = source;

        let runtime = EmRuntime::from_ui(&state, &identity_grid());
        let value = runtime.magnetic_at(
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            0.0,
        );

        assert_vector_close(value, expected_b, "direct E derived B");
    }
}

#[test]
fn em_runtime_electric_source_derives_opposite_direction_plane_wave_sign() {
    let mut state = EmUiState::default();
    state.mode = EmMode::Electric;
    state.electric_field = exprs("0", "cos(z + t)", "0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let value = runtime.magnetic_at(
        Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        0.0,
    );

    assert_vector_close(value, vector![1.0, 0.0, 0.0], "opposite direction B");
}

#[test]
fn em_runtime_electric_source_plane_wave_satisfies_maxwell_residuals() {
    let mut state = EmUiState::default();
    state.mode = EmMode::Electric;
    state.light_speed = 2.0;
    state.electric_field = exprs("0", "cos(z - 2*t)", "0");

    let runtime = EmRuntime::from_ui(&state, &identity_grid());
    let point = Point {
        x: 0.0,
        y: 0.0,
        z: 0.3,
    };
    let time = 0.2;
    let h = 1.0e-4;
    let curl_e = finite_curl(
        |point, time| runtime.electric_at(point, time),
        point,
        time,
        h,
    );
    let curl_b = finite_curl(
        |point, time| runtime.magnetic_at(point, time),
        point,
        time,
        h,
    );
    let partial_t_e = finite_partial_t(
        |point, time| runtime.electric_at(point, time),
        point,
        time,
        h,
    );
    let partial_t_b = finite_partial_t(
        |point, time| runtime.magnetic_at(point, time),
        point,
        time,
        h,
    );
    let div_b = finite_divergence(
        |point, time| runtime.magnetic_at(point, time),
        point,
        time,
        h,
    );

    assert_close_tol(div_b, 0.0, 1.0e-5, "div B");
    assert_close_tol((curl_e + partial_t_b).norm(), 0.0, 1.0e-5, "Faraday");
    assert_close_tol(
        (curl_b - partial_t_e / (state.light_speed * state.light_speed)).norm(),
        0.0,
        1.0e-5,
        "Ampere-Maxwell",
    );
}

#[test]
fn em_runtime_electric_source_preserves_standing_and_damped_inputs() {
    let cases = [
        (
            exprs("0", "cos(z - t) - cos(z + t)", "0"),
            0.75f64.cos() - 1.25f64.cos(),
        ),
        (
            exprs("0", "exp(-0.25*z) * cos(z - t)", "0"),
            (-0.25f64).exp() * 0.75f64.cos(),
        ),
    ];

    for (source, expected_y) in cases {
        let mut state = EmUiState::default();
        state.mode = EmMode::Electric;
        state.electric_field = source;
        let runtime = EmRuntime::from_ui(&state, &identity_grid());
        let value = runtime.electric_at(
            Point {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            0.25,
        );

        assert_close(value.x, 0.0, "preserved E x");
        assert_close(value.y, expected_y, "preserved E y");
        assert_close(value.z, 0.0, "preserved E z");
    }
}

#[test]
fn em_runtime_magnetic_source_derives_plane_wave_electric_field_for_each_axis() {
    let cases = [
        (exprs("0", "0", "cos(x - t)"), vector![0.0, 1.0, 0.0]),
        (exprs("cos(y - t)", "0", "0"), vector![0.0, 0.0, 1.0]),
        (exprs("cos(z - t)", "0", "0"), vector![0.0, -1.0, 0.0]),
    ];

    for (source, expected_e) in cases {
        let mut state = EmUiState::default();
        state.mode = EmMode::Magnetic;
        state.magnetic_field = source;

        let runtime = EmRuntime::from_ui(&state, &identity_grid());
        let value = runtime.electric_at(
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            0.0,
        );

        assert_vector_close(value, expected_e, "direct B derived E");
    }
}

#[test]
fn em_runtime_magnetic_source_preserves_standing_and_damped_inputs() {
    let cases = [
        (
            exprs("-cos(z - t) - cos(z + t)", "0", "0"),
            -0.75f64.cos() - 1.25f64.cos(),
        ),
        (
            exprs("exp(-0.25*z) * (0.25*sin(z - t) - cos(z - t))", "0", "0"),
            (-0.25f64).exp() * (0.25 * 0.75f64.sin() - 0.75f64.cos()),
        ),
    ];

    for (source, expected_x) in cases {
        let mut state = EmUiState::default();
        state.mode = EmMode::Magnetic;
        state.magnetic_field = source;
        let runtime = EmRuntime::from_ui(&state, &identity_grid());
        let value = runtime.magnetic_at(
            Point {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            0.25,
        );

        assert_close(value.x, expected_x, "preserved B x");
        assert_close(value.y, 0.0, "preserved B y");
        assert_close(value.z, 0.0, "preserved B z");
    }
}
