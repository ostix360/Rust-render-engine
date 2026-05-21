use super::fields::TimedVectorField;
use super::maxwell::{maxwell_inverse_curl, MaxwellSolveConfig};
use crate::app::coords_sys::CoordsSys;
use crate::app::grid::GridConfig;
use crate::maths::{FastExpr4d, Point};
use mathhook_core::Parser;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn counted_source() -> (TimedVectorField, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let count_eval = |value: f64| {
        let calls = calls.clone();
        Arc::new(move |_x, _y, _z, _t| {
            calls.fetch_add(1, Ordering::SeqCst);
            value
        }) as FastExpr4d
    };

    (
        TimedVectorField::from_fast_exprs([count_eval(1.0), count_eval(2.0), count_eval(3.0)]),
        calls,
    )
}

fn cache_test_config() -> MaxwellSolveConfig {
    let parse = |expr: &str| Parser::default().parse(expr).unwrap();
    let coords = CoordsSys::new(parse("x"), parse("y"), parse("z"));
    MaxwellSolveConfig::from_grid_config(
        GridConfig::new(0.0, 2.0, 2.0, 0.0, 2.0, 2.0, 0.0, 2.0, 2.0),
        coords.sample_geometry(),
    )
}

#[test]
fn inverse_curl_reuses_source_samples_per_time() {
    let (source, calls) = counted_source();
    let config = cache_test_config();
    let cell_count = config.cell_count();
    let field = maxwell_inverse_curl(source, config);

    let _ = field.at(
        Point {
            x: 0.25,
            y: 0.5,
            z: 0.75,
        },
        0.5,
    );
    let _ = field.at(
        Point {
            x: 1.25,
            y: 1.5,
            z: 1.75,
        },
        0.5,
    );

    assert_eq!(calls.load(Ordering::SeqCst), cell_count * 3);

    let _ = field.at(
        Point {
            x: 1.25,
            y: 1.5,
            z: 1.75,
        },
        0.75,
    );

    assert_eq!(calls.load(Ordering::SeqCst), cell_count * 6);
}

#[test]
fn inverse_curl_reuses_source_samples_per_time_for_parallel_targets() {
    let (source, calls) = counted_source();
    let config = cache_test_config();
    let cell_count = config.cell_count();
    let field = maxwell_inverse_curl(source, config);
    let points = [
        Point {
            x: 0.25,
            y: 0.5,
            z: 0.75,
        },
        Point {
            x: 1.25,
            y: 1.5,
            z: 1.75,
        },
        Point {
            x: 0.75,
            y: 1.25,
            z: 0.5,
        },
        Point {
            x: 1.75,
            y: 0.25,
            z: 1.5,
        },
    ];

    points.par_iter().for_each(|point| {
        let _ = field.at(*point, 0.5);
    });

    assert_eq!(calls.load(Ordering::SeqCst), cell_count * 3);
}
