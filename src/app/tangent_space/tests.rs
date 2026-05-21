use super::{
    compute_zoom_offset, requested_view, smoothstep, DiveAnchor, DiveMode, SceneSpaceTransform,
    TangentSpace, TangentView, DEFAULT_GEOMETRIC_ARROW_SCALE, DEFAULT_GEOMETRIC_LOCAL_SCALE,
    DUAL_FORM_GRID_RADIUS, DUAL_FORM_GRID_STEP,
};
use crate::toolbox::input::Input;
use glfw::{Action, Key};
use nalgebra::vector;

fn identity_anchor() -> DiveAnchor {
    DiveAnchor {
        abstract_pos: vector![1.0, 2.0, 3.0],
        world_pos: vector![0.0, 0.0, 0.0],
        basis: [
            vector![1.0, 0.0, 0.0],
            vector![0.0, 1.0, 0.0],
            vector![0.0, 0.0, 1.0],
        ],
        zoom_offset: vector![0.0, 0.0, 0.0],
    }
}

#[test]
fn tangent_anchor_maps_to_origin() {
    let anchor = identity_anchor();

    assert_eq!(
        anchor.geometric_tangent_position(anchor.abstract_pos, DEFAULT_GEOMETRIC_LOCAL_SCALE),
        vector![0.0, 0.0, 0.0]
    );
}

#[test]
fn abstract_delta_stays_unscaled_when_local_patch_size_changes() {
    let mut tangent_space = TangentSpace::new();
    tangent_space.dive.anchor = Some(identity_anchor());
    tangent_space.set_geometric_local_scale(0.1);

    assert_eq!(
        tangent_space.abstract_delta(vector![3.0, 2.0, 3.0]),
        Some(vector![2.0, 0.0, 0.0])
    );
    assert_eq!(
        tangent_space.geometric_local_delta(vector![3.0, 2.0, 3.0]),
        Some(vector![0.2, 0.0, 0.0])
    );
}

#[test]
fn local_sample_filter_uses_scaled_geometric_patch() {
    let mut tangent_space = TangentSpace::new();
    tangent_space.dive.mode = DiveMode::Tangent;
    tangent_space.dive.view = TangentView::Geometric;
    tangent_space.dive.anchor = Some(identity_anchor());
    tangent_space.set_geometric_local_scale(0.5);

    assert!(tangent_space.contains_local_sample(vector![8.0, 2.0, 3.0]));
    assert!(!tangent_space.contains_local_sample(vector![10.0, 2.0, 3.0]));
}

#[test]
fn camera_endpoints_apply_anchor_translation_and_zoom() {
    let anchor = DiveAnchor {
        abstract_pos: vector![0.0, 0.0, 0.0],
        world_pos: vector![4.0, 5.0, 6.0],
        basis: [
            vector![1.0, 0.0, 0.0],
            vector![0.0, 1.0, 0.0],
            vector![0.0, 0.0, 1.0],
        ],
        zoom_offset: vector![0.5, 0.0, 0.0],
    };

    let endpoints = anchor.build_camera_endpoints(vector![8.0, 9.0, 10.0]);

    assert_eq!(endpoints.world_pos, vector![8.0, 9.0, 10.0]);
    assert_eq!(endpoints.tangent_pos, vector![4.5, 4.0, 4.0]);
}

#[test]
fn zoom_offset_is_clamped_without_overshooting() {
    assert_eq!(
        compute_zoom_offset(vector![0.0, 0.0, 0.0], vector![10.0, 0.0, 0.0]),
        vector![8.0, 0.0, 0.0]
    );
    assert_eq!(
        compute_zoom_offset(vector![0.0, 0.0, 0.0], vector![1.0, 0.0, 0.0]),
        vector![0.8, 0.0, 0.0]
    );
}

#[test]
fn smoothstep_respects_endpoints() {
    assert_eq!(smoothstep(0.0), 0.0);
    assert_eq!(smoothstep(1.0), 1.0);
}

#[test]
fn dive_mode_animation_flags_are_scoped() {
    assert!(!DiveMode::World.is_animating());
    assert!(DiveMode::Entering.is_animating());
    assert!(!DiveMode::Tangent.is_animating());
    assert!(DiveMode::Exiting.is_animating());
}

#[test]
fn identity_scene_transform_has_zero_mix() {
    let transform = SceneSpaceTransform::identity();

    assert_eq!(transform.tangent_mix, 0.0);
    assert_eq!(
        transform.tangent_position_scale,
        DEFAULT_GEOMETRIC_LOCAL_SCALE
    );
}

#[test]
fn tangent_space_defaults_to_expected_arrow_scale() {
    let tangent_space = TangentSpace::new();

    assert_eq!(
        tangent_space.geometric_arrow_scale,
        DEFAULT_GEOMETRIC_ARROW_SCALE
    );
}

#[test]
fn requested_view_uses_t_and_ctrl_t() {
    let mut input = Input::new();
    input.begin_frame();
    input.key_handler(Action::Press, Key::T);
    assert_eq!(requested_view(&input), Some(TangentView::Geometric));

    let mut input = Input::new();
    input.begin_frame();
    input.key_handler(Action::Press, Key::LeftControl);
    input.key_handler(Action::Press, Key::T);
    assert_eq!(requested_view(&input), Some(TangentView::Dual));
}

#[test]
fn dual_view_hides_grid_once_transition_completes() {
    let mut tangent_space = TangentSpace::new();
    tangent_space.dive.mode = DiveMode::Tangent;
    tangent_space.dive.alpha = 1.0;
    tangent_space.dive.view = TangentView::Dual;

    assert!(!tangent_space.show_grid());
    assert!(tangent_space.show_form_samples());
    assert!(!tangent_space.show_vector_field());
}

#[test]
fn geometric_tangent_vector_uses_anchor_basis_orientation() {
    let anchor = DiveAnchor {
        abstract_pos: vector![0.0, 0.0, 0.0],
        world_pos: vector![0.0, 0.0, 0.0],
        basis: [
            vector![0.0, 0.0, 1.0],
            vector![1.0, 0.0, 0.0],
            vector![0.0, 1.0, 0.0],
        ],
        zoom_offset: vector![0.0, 0.0, 0.0],
    };

    assert_eq!(
        anchor.geometric_tangent_vector(vector![1.0, 0.0, 0.0]),
        vector![0.0, 0.0, 1.0]
    );
}

#[test]
fn dual_form_render_uses_anchor_basis_for_sample_positions() {
    let mut tangent_space = TangentSpace::new();
    tangent_space.dive.anchor = Some(DiveAnchor {
        abstract_pos: vector![0.0, 0.0, 0.0],
        world_pos: vector![0.0, 0.0, 0.0],
        basis: [
            vector![0.0, 0.0, 1.0],
            vector![1.0, 0.0, 0.0],
            vector![0.0, 1.0, 0.0],
        ],
        zoom_offset: vector![0.0, 0.0, 0.0],
    });

    let render = tangent_space
        .build_dual_form_render(vector![1.0, 0.0, 0.0])
        .expect("dual render");

    let expected = vector![0.0, 0.0, DUAL_FORM_GRID_STEP * DUAL_FORM_GRID_RADIUS as f64];
    assert!(render
        .samples
        .iter()
        .any(|sample| sample.position == expected));
}
