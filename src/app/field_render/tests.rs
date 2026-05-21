use super::{
    build_scalar_render, build_scalar_render_with_kind, build_vector_render_with_color,
    em_cache::time_normalization_scale, normalized_or_original, FieldSample, VectorNormalization,
    VectorRenderConfig,
};
use crate::app::tangent_space::TangentSpace;
use crate::app::ui::LegendKind;
use nalgebra::{vector, Vector3, Vector4};

fn origin_sample() -> FieldSample {
    FieldSample {
        abstract_pos: vector![0.0, 0.0, 0.0],
        world_pos: vector![0.0, 0.0, 0.0],
        basis: [
            vector![1.0, 0.0, 0.0],
            vector![0.0, 1.0, 0.0],
            vector![0.0, 0.0, 1.0],
        ],
    }
}

#[test]
fn normalization_helper_preserves_direction_and_unit_length() {
    let normalized = normalized_or_original(vector![0.0, 0.0, -4.0]);

    assert_eq!(normalized, vector![0.0, 0.0, -1.0]);
}

#[test]
fn normalization_helper_leaves_zero_vector_unchanged() {
    let normalized = normalized_or_original(vector![0.0, 0.0, 0.0]);

    assert_eq!(normalized, vector![0.0, 0.0, 0.0]);
}

#[test]
fn vector_and_scalar_renderables_can_be_assembled_together() {
    let samples = vec![origin_sample()];
    let tangent_space = TangentSpace::new();
    let vectors = build_vector_render_with_color(
        &samples,
        &[Vector3::new(1.0, 0.0, 0.0)],
        &[Vector3::new(1.0, 0.0, 0.0)],
        &tangent_space,
        VectorRenderConfig {
            normalization: VectorNormalization::None,
        },
        Vector4::new(0.0, 0.8, 1.0, 1.0),
    );
    let scalars = build_scalar_render(&samples, &[1.0], &tangent_space, 0.1);

    assert_eq!(vectors.len(), 1);
    assert_eq!(scalars.samples.len(), 1);
}

#[test]
fn scalar_potential_render_uses_potential_legend_and_value_colors() {
    let samples = vec![
        origin_sample(),
        FieldSample {
            abstract_pos: vector![1.0, 0.0, 0.0],
            world_pos: vector![1.0, 0.0, 0.0],
            ..origin_sample()
        },
    ];

    let render = build_scalar_render_with_kind(
        &samples,
        &[0.0, 10.0],
        &TangentSpace::new(),
        0.1,
        LegendKind::ScalarPotential,
    );

    let legend = render.legend.expect("expected scalar potential legend");
    assert_eq!(legend.kind, LegendKind::ScalarPotential);
    assert_eq!(legend.min_value, 0.0);
    assert_eq!(legend.max_value, 10.0);
    assert_ne!(render.samples[0].get_color(), render.samples[1].get_color());
}

#[test]
fn time_normalization_scale_uses_each_vectors_temporal_amplitude() {
    let scale =
        time_normalization_scale(&origin_sample(), 0.0, Vector3::new(0.0, 0.0, 0.0), |time| {
            Vector3::new(0.0, 4.0 * time.sin(), 0.0)
        });

    assert!((scale - 0.25).abs() < 1.0e-6);
}

#[test]
fn time_normalization_scale_includes_current_time_for_non_periodic_fields() {
    let scale = time_normalization_scale(
        &origin_sample(),
        10.0,
        Vector3::new(10.0, 0.0, 0.0),
        |time| Vector3::new(time.abs(), 0.0, 0.0),
    );

    assert!(scale <= 0.1);
}
