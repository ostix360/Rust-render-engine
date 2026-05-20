//! Electromagnetism runtime fields over animated 3D slices.

use crate::app::grid::{Grid, GridConfig};
use crate::app::ui::{EmGauge, EmLayerVisibility, EmMode, EmUiState, SpacialEqs};
use crate::maths::differential::Form;
use crate::maths::space::Space;
use crate::maths::{derivate, expr_to_fastexpr4d, Expr, ExternalDerivative, FastExpr4d, Point};
use mathhook_core::Simplify;
use nalgebra::Vector3;
use std::ops::{Add, Mul};
use std::sync::Arc;

const MAXWELL_MIN_AXIS_SAMPLES: usize = 5;
const MAXWELL_MAX_AXIS_SAMPLES: usize = 7;
const MAXWELL_SINGULAR_EPSILON_SQUARED: f64 = 1.0e-18;
const PLANE_WAVE_EPSILON: f64 = 1.0e-7;

#[derive(Clone, Copy)]
struct MaxwellCell {
    point: Point,
    weight: f64,
}

#[derive(Clone)]
struct MaxwellSolveConfig {
    cells: Arc<[MaxwellCell]>,
}

impl MaxwellSolveConfig {
    fn from_grid_config(grid_config: GridConfig) -> Self {
        let bounds = grid_config.bounds();
        let counts = grid_config.sample_counts().map(Self::axis_sample_count);
        let normalized_bounds = bounds.map(Self::normalized_bounds);
        let steps = [
            (normalized_bounds[0].1 - normalized_bounds[0].0) / counts[0] as f64,
            (normalized_bounds[1].1 - normalized_bounds[1].0) / counts[1] as f64,
            (normalized_bounds[2].1 - normalized_bounds[2].0) / counts[2] as f64,
        ];
        let weight = steps[0] * steps[1] * steps[2];
        let mut cells = Vec::with_capacity(counts[0] * counts[1] * counts[2]);

        for ix in 0..counts[0] {
            let x = normalized_bounds[0].0 + (ix as f64 + 0.5) * steps[0];
            for iy in 0..counts[1] {
                let y = normalized_bounds[1].0 + (iy as f64 + 0.5) * steps[1];
                for iz in 0..counts[2] {
                    let z = normalized_bounds[2].0 + (iz as f64 + 0.5) * steps[2];
                    cells.push(MaxwellCell {
                        point: Point { x, y, z },
                        weight,
                    });
                }
            }
        }

        Self {
            cells: Arc::from(cells),
        }
    }

    fn axis_sample_count(count: usize) -> usize {
        count.clamp(MAXWELL_MIN_AXIS_SAMPLES, MAXWELL_MAX_AXIS_SAMPLES)
    }

    fn normalized_bounds((min, max): (f64, f64)) -> (f64, f64) {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        if (max - min).abs() <= f64::EPSILON {
            (min - 0.5, max + 0.5)
        } else {
            (min, max)
        }
    }
}

#[derive(Clone)]
pub struct TimedScalarField {
    fast_expr: FastExpr4d,
}

impl TimedScalarField {
    pub fn new(expr: Expr) -> Self {
        Self {
            fast_expr: expr_to_fastexpr4d(expr),
        }
    }

    pub fn from_fast_expr(fast_expr: FastExpr4d) -> Self {
        Self { fast_expr }
    }

    pub fn at(&self, point: Point, time: f64) -> f64 {
        (self.fast_expr)(point.x, point.y, point.z, time)
    }
}

#[derive(Clone)]
pub struct TimedVectorField {
    fast_exprs: [FastExpr4d; 3],
}

impl TimedVectorField {
    pub fn from_exprs(exprs: [Expr; 3]) -> Self {
        Self {
            fast_exprs: [
                expr_to_fastexpr4d(exprs[0].clone()),
                expr_to_fastexpr4d(exprs[1].clone()),
                expr_to_fastexpr4d(exprs[2].clone()),
            ],
        }
    }

    pub fn from_fast_exprs(fast_exprs: [FastExpr4d; 3]) -> Self {
        Self { fast_exprs }
    }

    pub fn at(&self, point: Point, time: f64) -> Vector3<f64> {
        Vector3::new(
            (self.fast_exprs[0])(point.x, point.y, point.z, time),
            (self.fast_exprs[1])(point.x, point.y, point.z, time),
            (self.fast_exprs[2])(point.x, point.y, point.z, time),
        )
    }
}

#[derive(Clone)]
pub struct EmRuntime {
    pub layers: EmLayerVisibility,
    magnetic_vector_scale: f64,
    phi: TimedScalarField,
    vector_potential: TimedVectorField,
    electric_field: TimedVectorField,
    magnetic_field: TimedVectorField,
}

impl EmRuntime {
    #[allow(dead_code)]
    pub fn from_ui(state: &EmUiState, grid: &Grid) -> Self {
        Self::from_ui_with_config(state, grid, GridConfig::default())
    }

    pub fn from_ui_with_config(state: &EmUiState, grid: &Grid, grid_config: GridConfig) -> Self {
        let maxwell_config = MaxwellSolveConfig::from_grid_config(grid_config);
        match state.mode {
            EmMode::Potentials => Self::from_potentials(state, grid.get_coords().get_space()),
            EmMode::Electric => Self::from_electric(state, maxwell_config),
            EmMode::Magnetic => Self::from_magnetic(state, maxwell_config),
        }
    }

    fn from_potentials(state: &EmUiState, space: &Space) -> Self {
        let phi_expr = state.phi.eq.clone();
        let a_otn_exprs = exprs_from_spacial(&state.vector_potential);
        let a_otn = Form::new_otn(a_otn_exprs.to_vec(), 1);
        let a_natural = a_otn.to_dual_base(space);

        let mut phi_form = Form::new(vec![phi_expr.clone()], 0);
        let grad_phi = phi_form.d();
        let electric_natural = Form::new(
            (0..3)
                .map(|index| {
                    negate(grad_phi.get_expr(index).clone())
                        .add(negate(partial_t(a_natural.get_expr(index).clone())))
                        .simplify()
                })
                .collect(),
            1,
        );
        let electric_otn = electric_natural.to_otn_base(space);
        let mut a_for_d = a_natural;
        let magnetic_otn = a_for_d.d().to_otn_base(space).hodge_star_otn_3d();

        Self {
            layers: state.layers.clone(),
            magnetic_vector_scale: state.magnetic_vector_scale,
            phi: TimedScalarField::new(phi_expr),
            vector_potential: TimedVectorField::from_exprs(a_otn_exprs),
            electric_field: TimedVectorField::from_exprs(form_exprs(&electric_otn)),
            magnetic_field: TimedVectorField::from_exprs(form_exprs(&magnetic_otn)),
        }
    }

    fn from_electric(state: &EmUiState, maxwell_config: MaxwellSolveConfig) -> Self {
        let electric_exprs = exprs_from_spacial(&state.electric_field);
        let c = state.light_speed.max(1.0e-6);

        let electric_field = TimedVectorField::from_exprs(electric_exprs.clone());
        let magnetic_field =
            if let Some(magnetic_exprs) = plane_wave_magnetic_exprs(&electric_exprs, c) {
                TimedVectorField::from_exprs(magnetic_exprs)
            } else {
                let ampere_source_exprs = maxwell_ampere_source_exprs(&electric_exprs, c);
                let ampere_source = TimedVectorField::from_exprs(ampere_source_exprs);
                maxwell_inverse_curl(ampere_source, maxwell_config)
            };
        let vector_potential = local_vector_potential_from_b(magnetic_field.clone());
        let phi = scalar_potential_for_gauge(state.gauge, electric_field.clone());

        Self {
            layers: state.layers.clone(),
            magnetic_vector_scale: state.magnetic_vector_scale,
            phi,
            vector_potential,
            electric_field,
            magnetic_field,
        }
    }

    fn from_magnetic(state: &EmUiState, maxwell_config: MaxwellSolveConfig) -> Self {
        let magnetic_exprs = exprs_from_spacial(&state.magnetic_field);
        let c = state.light_speed.max(1.0e-6);

        let magnetic_field = TimedVectorField::from_exprs(magnetic_exprs.clone());
        let electric_field =
            if let Some(electric_exprs) = plane_wave_electric_exprs(&magnetic_exprs, c) {
                TimedVectorField::from_exprs(electric_exprs)
            } else {
                let faraday_source_exprs = maxwell_faraday_source_exprs(&magnetic_exprs);
                let faraday_source = TimedVectorField::from_exprs(faraday_source_exprs);
                maxwell_inverse_curl(faraday_source, maxwell_config.clone())
            };
        let phi = scalar_potential_for_gauge(state.gauge, electric_field.clone());
        let vector_potential = maxwell_inverse_curl(magnetic_field.clone(), maxwell_config);

        Self {
            layers: state.layers.clone(),
            magnetic_vector_scale: state.magnetic_vector_scale,
            phi,
            vector_potential,
            electric_field,
            magnetic_field,
        }
    }

    pub fn phi_at(&self, point: Point, time: f64) -> f64 {
        self.phi.at(point, time)
    }

    pub fn vector_potential_at(&self, point: Point, time: f64) -> Vector3<f64> {
        self.vector_potential.at(point, time)
    }

    pub fn electric_at(&self, point: Point, time: f64) -> Vector3<f64> {
        self.electric_field.at(point, time)
    }

    pub fn magnetic_at(&self, point: Point, time: f64) -> Vector3<f64> {
        self.magnetic_field.at(point, time)
    }

    pub fn magnetic_render_scale(&self) -> f64 {
        self.magnetic_vector_scale
    }

    pub fn active_layers(&self) -> EmLayerVisibility {
        self.layers.clone()
    }

    pub fn active_vector_layer_count(&self) -> usize {
        let layers = self.active_layers();
        usize::from(layers.electric)
            + usize::from(layers.magnetic)
            + usize::from(layers.vector_potential)
    }
}

fn scale_exprs(exprs: [Expr; 3], scale: f64) -> [Expr; 3] {
    [
        scale_expr(exprs[0].clone(), scale),
        scale_expr(exprs[1].clone(), scale),
        scale_expr(exprs[2].clone(), scale),
    ]
}

fn scale_expr(expr: Expr, scale: f64) -> Expr {
    Expr::number(scale).mul(expr).simplify()
}

fn maxwell_ampere_source_exprs(electric_exprs: &[Expr; 3], c: f64) -> [Expr; 3] {
    scale_exprs(partial_t_exprs(electric_exprs), 1.0 / (c * c))
}

fn maxwell_faraday_source_exprs(magnetic_exprs: &[Expr; 3]) -> [Expr; 3] {
    scale_exprs(partial_t_exprs(magnetic_exprs), -1.0)
}

fn plane_wave_magnetic_exprs(electric_exprs: &[Expr; 3], c: f64) -> Option<[Expr; 3]> {
    if !electric_exprs
        .iter()
        .any(|expr| !is_near_zero_expr(&partial_t(expr.clone())))
    {
        return None;
    }

    for axis in 0..3 {
        if !is_near_zero_expr(&electric_exprs[axis]) {
            continue;
        }

        for direction in [1.0, -1.0] {
            if is_plane_wave_direction(electric_exprs, axis, c, direction) {
                return Some(scale_exprs(
                    cross_axis_exprs(axis, direction, electric_exprs),
                    1.0 / c,
                ));
            }
        }
    }

    None
}

fn plane_wave_electric_exprs(magnetic_exprs: &[Expr; 3], c: f64) -> Option<[Expr; 3]> {
    if !magnetic_exprs
        .iter()
        .any(|expr| !is_near_zero_expr(&partial_t(expr.clone())))
    {
        return None;
    }

    for axis in 0..3 {
        if !is_near_zero_expr(&magnetic_exprs[axis]) {
            continue;
        }

        for direction in [1.0, -1.0] {
            if is_plane_wave_direction(magnetic_exprs, axis, c, direction) {
                return Some(scale_exprs(
                    cross_axis_exprs(axis, direction, magnetic_exprs),
                    -c,
                ));
            }
        }
    }

    None
}

fn is_plane_wave_direction(exprs: &[Expr; 3], axis: usize, c: f64, direction: f64) -> bool {
    exprs.iter().all(|expr| {
        (0..3)
            .filter(|sample_axis| *sample_axis != axis)
            .all(|sample_axis| is_near_zero_expr(&derivate_axis(expr.clone(), sample_axis)))
            && {
                let wave_residual = partial_t(expr.clone())
                    .add(scale_expr(derivate_axis(expr.clone(), axis), direction * c))
                    .simplify();
                is_near_zero_expr(&wave_residual)
            }
    })
}

fn derivate_axis(expr: Expr, axis: usize) -> Expr {
    derivate(expr, &axis_name(axis).to_string())
}

fn axis_name(axis: usize) -> &'static str {
    match axis {
        0 => "x",
        1 => "y",
        2 => "z",
        _ => panic!("plane wave axis must be 0, 1, or 2"),
    }
}

fn cross_axis_exprs(axis: usize, direction: f64, rhs: &[Expr; 3]) -> [Expr; 3] {
    let zero = Expr::number(0.0);
    let signed = |expr: Expr, sign: f64| scale_expr(expr, direction * sign);

    match axis {
        0 => [
            zero,
            signed(rhs[2].clone(), -1.0),
            signed(rhs[1].clone(), 1.0),
        ],
        1 => [
            signed(rhs[2].clone(), 1.0),
            zero,
            signed(rhs[0].clone(), -1.0),
        ],
        2 => [
            signed(rhs[1].clone(), -1.0),
            signed(rhs[0].clone(), 1.0),
            zero,
        ],
        _ => panic!("plane wave axis must be 0, 1, or 2"),
    }
}

fn is_near_zero_expr(expr: &Expr) -> bool {
    let evaluator = expr_to_fastexpr4d(expr.clone());
    let samples = [
        (-1.0, -0.5, -0.25, 0.0),
        (-0.25, 0.75, 0.5, 0.3),
        (0.5, -0.75, 1.0, -0.8),
        (1.0, 0.25, -1.0, 1.2),
    ];

    samples.into_iter().all(|(x, y, z, t)| {
        let value = evaluator(x, y, z, t);
        value.is_finite() && value.abs() <= PLANE_WAVE_EPSILON
    })
}

fn partial_t_exprs(exprs: &[Expr; 3]) -> [Expr; 3] {
    [
        partial_t(exprs[0].clone()).simplify(),
        partial_t(exprs[1].clone()).simplify(),
        partial_t(exprs[2].clone()).simplify(),
    ]
}

/// Reconstructs a divergence-free vector field whose curl matches `source`.
///
/// This uses the Coulomb-gauge Biot-Savart inverse curl over the active finite grid volume.
/// The assumption is intentionally explicit here: direct `E` and `B` source modes need a
/// boundary/gauge choice before the complementary Maxwell field is determined.
fn maxwell_inverse_curl(source: TimedVectorField, config: MaxwellSolveConfig) -> TimedVectorField {
    let source_exprs = source.fast_exprs;

    TimedVectorField::from_fast_exprs([
        maxwell_inverse_curl_component(
            [
                source_exprs[0].clone(),
                source_exprs[1].clone(),
                source_exprs[2].clone(),
            ],
            config.clone(),
            0,
        ),
        maxwell_inverse_curl_component(
            [
                source_exprs[0].clone(),
                source_exprs[1].clone(),
                source_exprs[2].clone(),
            ],
            config.clone(),
            1,
        ),
        maxwell_inverse_curl_component(source_exprs, config, 2),
    ])
}

fn maxwell_inverse_curl_component(
    source: [FastExpr4d; 3],
    config: MaxwellSolveConfig,
    component: usize,
) -> FastExpr4d {
    Arc::new(move |x, y, z, t| {
        let target = Vector3::new(x, y, z);
        let mut value = 0.0;

        for cell in config.cells.iter() {
            let source_point = Vector3::new(cell.point.x, cell.point.y, cell.point.z);
            let radius = target - source_point;
            let radius_squared = radius.norm_squared();
            if radius_squared <= MAXWELL_SINGULAR_EPSILON_SQUARED {
                continue;
            }

            let source_value = Vector3::new(
                (source[0])(cell.point.x, cell.point.y, cell.point.z, t),
                (source[1])(cell.point.x, cell.point.y, cell.point.z, t),
                (source[2])(cell.point.x, cell.point.y, cell.point.z, t),
            );
            if !source_value.x.is_finite()
                || !source_value.y.is_finite()
                || !source_value.z.is_finite()
            {
                continue;
            }

            let kernel_scale =
                cell.weight / (4.0 * std::f64::consts::PI * radius_squared.sqrt().powi(3));
            value += source_value.cross(&radius)[component] * kernel_scale;
        }

        value
    })
}

/// Builds a cheap local visualization potential from a magnetic field sample.
///
/// In electric source mode `B` is already a finite-domain inverse-curl field. Running the same
/// inverse-curl solver again for `A` would nest the quadrature and make the default visible
/// `A` layer quadratic in the Maxwell cell count. This local Coulomb-style potential keeps the
/// layer responsive while preserving a meaningful nonzero vector-potential visualization.
fn local_vector_potential_from_b(magnetic_field: TimedVectorField) -> TimedVectorField {
    let bx = magnetic_field.fast_exprs[0].clone();
    let by = magnetic_field.fast_exprs[1].clone();
    let bz = magnetic_field.fast_exprs[2].clone();

    TimedVectorField::from_fast_exprs([
        {
            let by = by.clone();
            let bz = bz.clone();
            Arc::new(move |x, y, z, t| 0.5 * (by(x, y, z, t) * z - bz(x, y, z, t) * y))
        },
        {
            let bx = bx.clone();
            let bz = bz.clone();
            Arc::new(move |x, y, z, t| 0.5 * (bz(x, y, z, t) * x - bx(x, y, z, t) * z))
        },
        Arc::new(move |x, y, z, t| 0.5 * (bx(x, y, z, t) * y - by(x, y, z, t) * x)),
    ])
}

fn local_scalar_potential_from_e(electric_field: TimedVectorField) -> TimedScalarField {
    let ex = electric_field.fast_exprs[0].clone();
    let ey = electric_field.fast_exprs[1].clone();
    let ez = electric_field.fast_exprs[2].clone();

    TimedScalarField::from_fast_expr(Arc::new(move |x, y, z, t| {
        -(ex(x, y, z, t) * x + ey(x, y, z, t) * y + ez(x, y, z, t) * z)
    }))
}

fn scalar_potential_for_gauge(
    gauge: EmGauge,
    electric_field: TimedVectorField,
) -> TimedScalarField {
    match gauge {
        EmGauge::Coulomb => local_scalar_potential_from_e(electric_field),
        // A zero Lorenz scalar potential would require a matching transform of `A`; otherwise
        // the displayed potentials no longer reconstruct the displayed source field.
        EmGauge::Lorenz => local_scalar_potential_from_e(electric_field),
    }
}

fn exprs_from_spacial(eqs: &SpacialEqs) -> [Expr; 3] {
    [eqs.x.eq.clone(), eqs.y.eq.clone(), eqs.z.eq.clone()]
}

fn form_exprs(form: &Form) -> [Expr; 3] {
    [
        form.get_expr(0).clone(),
        form.get_expr(1).clone(),
        form.get_expr(2).clone(),
    ]
}

fn partial_t(expr: Expr) -> Expr {
    derivate(expr, &"t".to_string())
}

fn negate(expr: Expr) -> Expr {
    Expr::number(-1.0).mul(expr)
}

#[cfg(test)]
mod tests {
    use super::EmRuntime;
    use crate::app::coords_sys::CoordsSys;
    use crate::app::grid::Grid;
    use crate::app::ui::{EmGauge, EmMode, EmUiState};
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
    fn potential_mode_preserves_static_curl_added_to_plane_wave_potential() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let mut state = EmUiState::default();
        state.mode = EmMode::Potentials;
        state.phi.eq = parse("0");
        state.vector_potential.x.eq = parse("0");
        state.vector_potential.y.eq = parse("sin(z - t) + x");
        state.vector_potential.z.eq = parse("0");

        let runtime = EmRuntime::from_ui(&state, &identity_grid());
        let value = runtime.magnetic_at(
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            0.0,
        );

        assert_close(value.x, -1.0);
        assert_near_zero(value.y);
        assert_close(value.z, 1.0);
    }

    #[test]
    fn electric_source_preserves_e_and_resolves_other_measures() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let mut state = EmUiState::default();
        state.mode = EmMode::Electric;
        state.electric_field.x.eq = parse("t");
        state.electric_field.y.eq = parse("x + y");
        state.electric_field.z.eq = parse("z");

        let runtime = EmRuntime::from_ui(&state, &identity_grid());
        let point = Point {
            x: 2.0,
            y: 3.0,
            z: 4.0,
        };

        assert_close(runtime.electric_at(point, 7.0).x, 7.0);
        assert_close(runtime.electric_at(point, 7.0).y, 5.0);
        assert!(runtime.vector_potential_at(point, 7.0).x.is_finite());
        assert!(runtime.magnetic_at(point, 7.0).z.is_finite());
    }

    #[test]
    fn electric_source_static_spatial_curl_does_not_create_magnetic_field() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let mut state = EmUiState::default();
        state.mode = EmMode::Electric;
        state.electric_field.x.eq = parse("0");
        state.electric_field.y.eq = parse("x");
        state.electric_field.z.eq = parse("0");

        let runtime = EmRuntime::from_ui(&state, &identity_grid());
        let value = runtime.magnetic_at(
            Point {
                x: 2.0,
                y: 3.0,
                z: 4.0,
            },
            0.0,
        );

        assert_near_zero(value.norm());
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
    fn electric_source_mixed_static_term_does_not_use_plane_wave_shortcut() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let electric_exprs = [parse("0"), parse("cos(z - t) + x"), parse("0")];

        assert!(super::plane_wave_magnetic_exprs(&electric_exprs, 1.0).is_none());
    }

    #[test]
    fn electric_source_derived_magnetic_field_is_divergence_free() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let mut state = EmUiState::default();
        state.mode = EmMode::Electric;
        state.electric_field.x.eq = parse("0");
        state.electric_field.y.eq = parse("t");
        state.electric_field.z.eq = parse("0");

        let runtime = EmRuntime::from_ui(&state, &identity_grid());
        let point = Point {
            x: 2.3,
            y: 1.7,
            z: 4.1,
        };
        let h = 1.0e-3;
        let dx = (runtime
            .magnetic_at(
                Point {
                    x: point.x + h,
                    ..point
                },
                0.0,
            )
            .x
            - runtime
                .magnetic_at(
                    Point {
                        x: point.x - h,
                        ..point
                    },
                    0.0,
                )
                .x)
            / (2.0 * h);
        let dy = (runtime
            .magnetic_at(
                Point {
                    y: point.y + h,
                    ..point
                },
                0.0,
            )
            .y
            - runtime
                .magnetic_at(
                    Point {
                        y: point.y - h,
                        ..point
                    },
                    0.0,
                )
                .y)
            / (2.0 * h);
        let dz = (runtime
            .magnetic_at(
                Point {
                    z: point.z + h,
                    ..point
                },
                0.0,
            )
            .z
            - runtime
                .magnetic_at(
                    Point {
                        z: point.z - h,
                        ..point
                    },
                    0.0,
                )
                .z)
            / (2.0 * h);

        assert_close_tol(dx + dy + dz, 0.0, 1.0e-5);
    }

    #[test]
    fn electric_source_reconstructs_nonzero_scalar_potential() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let mut state = EmUiState::default();
        state.mode = EmMode::Electric;
        state.electric_field.x.eq = parse("2");
        state.electric_field.y.eq = parse("0");
        state.electric_field.z.eq = parse("0");

        let runtime = EmRuntime::from_ui(&state, &identity_grid());

        assert_close(
            runtime.phi_at(
                Point {
                    x: 3.0,
                    y: 0.0,
                    z: 0.0,
                },
                0.0,
            ),
            -6.0,
        );
    }

    #[test]
    fn lorenz_source_gauge_keeps_potential_reconstruction_consistent() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let point = Point {
            x: 3.0,
            y: 0.0,
            z: 0.0,
        };
        let mut state = EmUiState::default();
        state.mode = EmMode::Electric;
        state.electric_field.x.eq = parse("2");
        state.electric_field.y.eq = parse("0");
        state.electric_field.z.eq = parse("0");

        state.gauge = EmGauge::Coulomb;
        let coulomb = EmRuntime::from_ui(&state, &identity_grid());
        state.gauge = EmGauge::Lorenz;
        let lorenz = EmRuntime::from_ui(&state, &identity_grid());

        assert_close(coulomb.phi_at(point, 0.0), -6.0);
        assert_close(lorenz.phi_at(point, 0.0), -6.0);
        assert_close(
            (coulomb.electric_at(point, 0.0) - lorenz.electric_at(point, 0.0)).norm(),
            0.0,
        );
    }

    #[test]
    fn electric_source_magnetic_field_does_not_grow_with_elapsed_time() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let mut state = EmUiState::default();
        state.mode = EmMode::Electric;
        state.electric_field.x.eq = parse("0");
        state.electric_field.y.eq = parse("x * sin(t)");
        state.electric_field.z.eq = parse("0");

        let runtime = EmRuntime::from_ui(&state, &identity_grid());
        let point = Point {
            x: 2.0,
            y: 0.0,
            z: 0.0,
        };

        let reference = runtime.magnetic_at(point, 0.0).norm().max(1.0e-9);
        let early = runtime.magnetic_at(point, 1.0).norm();
        let late = runtime.magnetic_at(point, 100.0).norm();

        assert!(early <= reference);
        assert!(late <= reference);
    }

    #[test]
    fn magnetic_source_preserves_b_and_resolves_other_measures() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let mut state = EmUiState::default();
        state.mode = EmMode::Magnetic;
        state.magnetic_field.x.eq = parse("1");
        state.magnetic_field.y.eq = parse("2");
        state.magnetic_field.z.eq = parse("3");

        let runtime = EmRuntime::from_ui(&state, &identity_grid());
        let point = Point {
            x: 2.0,
            y: 3.0,
            z: 4.0,
        };

        assert_close(runtime.magnetic_at(point, 7.0).z, 3.0);
        assert!(runtime.vector_potential_at(point, 7.0).x.is_finite());
        assert_near_zero(runtime.electric_at(point, 7.0).norm());
    }

    #[test]
    fn magnetic_source_static_spatial_curl_does_not_create_electric_field() {
        let parse = |expr: &str| Parser::default().parse(expr).unwrap();
        let mut state = EmUiState::default();
        state.mode = EmMode::Magnetic;
        state.magnetic_field.x.eq = parse("0");
        state.magnetic_field.y.eq = parse("x");
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

        assert_near_zero(value.norm());
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
}
