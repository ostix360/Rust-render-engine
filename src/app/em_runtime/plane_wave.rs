use crate::maths::{derivate, Expr};
use mathhook_core::Simplify;
use std::ops::{Add, Mul};

pub(super) fn plane_wave_magnetic_exprs(electric_exprs: &[Expr; 3], c: f64) -> Option<[Expr; 3]> {
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

pub(super) fn plane_wave_electric_exprs(magnetic_exprs: &[Expr; 3], c: f64) -> Option<[Expr; 3]> {
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

pub(super) fn scale_exprs(exprs: [Expr; 3], scale: f64) -> [Expr; 3] {
    [
        scale_expr(exprs[0].clone(), scale),
        scale_expr(exprs[1].clone(), scale),
        scale_expr(exprs[2].clone(), scale),
    ]
}

fn scale_expr(expr: Expr, scale: f64) -> Expr {
    Expr::number(scale).mul(expr).simplify()
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

fn partial_t(expr: Expr) -> Expr {
    derivate(expr, &"t".to_string())
}

fn is_near_zero_expr(expr: &Expr) -> bool {
    expr.simplify().is_zero()
}
