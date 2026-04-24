use std::f64::consts::{FRAC_PI_2, PI};

use nalgebra::{
    vector, Matrix4, Perspective3, Rotation3, Translation3, UnitQuaternion, Vector3, Vector4,
};
use render_engine::toolbox::camera::Camera;

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

fn assert_matrix_close(actual: &Matrix4<f64>, expected: &Matrix4<f64>, context: &str) {
    for row in 0..4 {
        for col in 0..4 {
            let ctx = format!("{context} [{}][{}]", row, col);
            assert_close(actual[(row, col)], expected[(row, col)], &ctx);
        }
    }
}

#[test]
fn perspective_matrix_matches_expected_entries() {
    let aspect = 16.0 / 9.0;
    let fovy = 1.6;
    let near = 0.1;
    let far = 1250.0;

    let projection = Perspective3::new(aspect, fovy, near, far).to_homogeneous();
    let half_fovy: f64 = fovy / 2.0;
    let f = 1.0 / half_fovy.tan();

    let mut expected = Matrix4::<f64>::zeros();
    expected[(0, 0)] = f / aspect;
    expected[(1, 1)] = f;
    expected[(2, 2)] = -(far + near) / (far - near);
    expected[(2, 3)] = -(2.0 * far * near) / (far - near);
    expected[(3, 2)] = -1.0;

    assert_matrix_close(&projection, &expected, "projection");
}

#[test]
fn view_matrix_tracks_camera_pose() {
    let mut camera = Camera::new(vector![0.0, 0.0, 0.0]);
    camera.position = vector![3.0, -2.0, 5.0];
    camera.quat = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 6.0)
        * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), -PI / 8.0);

    let view = camera.get_view_matrix();
    let translation = Translation3::from(-camera.position).to_homogeneous();
    let rotation = camera.quat.inverse().to_homogeneous();
    let expected = rotation * translation;

    assert_matrix_close(&view, &expected, "view");
}

#[test]
fn model_view_projection_pipeline_maps_point_correctly() {
    let width = 1420.0;
    let height = 920.0;
    let aspect_ratio = width / height;
    let fovy = 1.6;
    let near = 0.1;
    let far = 1250.0;

    let projection = Perspective3::new(aspect_ratio, fovy, near, far).to_homogeneous();

    let mut camera = Camera::new(vector![0.0, 0.0, 0.0]);
    camera.position = vector![1.5, -0.5, 0.75];
    camera.quat =
        UnitQuaternion::from_axis_angle(&Vector3::y_axis(), FRAC_PI_2) * UnitQuaternion::identity();
    let view = camera.get_view_matrix();

    let translation = Translation3::from(Vector3::new(0.25, 0.4, -6.0)).to_homogeneous();
    let rotation = Rotation3::from_euler_angles(0.0, FRAC_PI_2, FRAC_PI_2).to_homogeneous();
    let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(1.5, 0.5, 0.75));
    let model = translation * rotation * scale;

    let local_point = Vector4::new(0.5, -0.25, 0.2, 1.0);
    let combined = projection * view * model;
    let clip = combined * local_point;
    let ndc_from_combined = Vector3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w);

    // Manual pipeline: S -> R -> T -> view -> projection, using explicit vector math.
    let scaled = Vector3::new(
        local_point.x * 1.5,
        local_point.y * 0.5,
        local_point.z * 0.75,
    );
    let rot_obj = Rotation3::from_euler_angles(0.0, FRAC_PI_2, FRAC_PI_2);
    let rotated = rot_obj * scaled;
    let world = rotated + Vector3::new(0.25, 0.4, -6.0);

    let camera_relative = world - camera.position;
    let eye = camera.quat.inverse() * camera_relative;

    let half_fovy = fovy / 2.0;
    let f = 1.0 / half_fovy.tan();
    let clip_manual = Vector4::new(
        (f / aspect_ratio) * eye.x,
        f * eye.y,
        -(far + near) / (far - near) * eye.z - (2.0 * far * near) / (far - near),
        -eye.z,
    );
    let ndc_manual = Vector3::new(
        clip_manual.x / clip_manual.w,
        clip_manual.y / clip_manual.w,
        clip_manual.z / clip_manual.w,
    );

    assert_close(
        ndc_from_combined.x,
        ndc_manual.x,
        "combined ndc x component mismatch",
    );
    assert_close(
        ndc_from_combined.y,
        ndc_manual.y,
        "combined ndc y component mismatch",
    );
    assert_close(
        ndc_from_combined.z,
        ndc_manual.z,
        "combined ndc z component mismatch",
    );
}
