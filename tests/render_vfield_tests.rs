use nalgebra::{vector, Vector4};
use render_engine::graphics::model::RenderVField;

const EPS: f64 = 1.0e-8;

fn assert_close(actual: f64, expected: f64, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= EPS,
        "{}: expected {:.12}, got {:.12} (delta {:.12})",
        context,
        expected,
        actual,
        delta
    );
}

#[test]
fn render_vfield_zero_vector_returns_zero_matrix() {
    let field = RenderVField::new(
        vector![1.0, 2.0, 3.0],
        vector![0.0, 0.0, 0.0],
        Vector4::new(1.0, 1.0, 0.0, 1.0),
    );

    let transform = field.get_transformation_matrix();

    assert!(transform.iter().all(|value| value.abs() <= EPS));
}

#[test]
fn render_vfield_transform_keeps_translation_at_position() {
    let field = RenderVField::new(
        vector![1.5, -2.0, 0.75],
        vector![0.0, 2.0, 0.0],
        Vector4::new(1.0, 1.0, 0.0, 1.0),
    );

    let transform = field.get_transformation_matrix();

    assert_close(transform[(0, 3)], 1.5, "translation x");
    assert_close(transform[(1, 3)], -2.0, "translation y");
    assert_close(transform[(2, 3)], 0.75, "translation z");
}

#[test]
fn render_vfield_transform_scales_with_vector_magnitude_along_y() {
    let field = RenderVField::new(
        vector![0.0, 0.0, 0.0],
        vector![0.0, 2.0, 0.0],
        Vector4::new(1.0, 1.0, 0.0, 1.0),
    );

    let transform = field.get_transformation_matrix();

    assert_close(transform[(0, 0)], 0.02, "radius scale x");
    assert_close(transform[(1, 1)], 0.4, "length scale y");
    assert_close(transform[(2, 2)], 0.02, "radius scale z");
}

#[test]
fn render_vfield_transform_rotates_arrow_toward_vector_direction() {
    let field = RenderVField::new(
        vector![0.0, 0.0, 0.0],
        vector![3.0, 0.0, 0.0],
        Vector4::new(1.0, 1.0, 0.0, 1.0),
    );

    let transform = field.get_transformation_matrix();
    let transformed_axis = transform * Vector4::new(0.0, 1.0, 0.0, 0.0);
    let direction = transformed_axis.xyz().normalize();

    assert_close(direction.x, 1.0, "direction x");
    assert_close(direction.y, 0.0, "direction y");
    assert_close(direction.z, 0.0, "direction z");
}
