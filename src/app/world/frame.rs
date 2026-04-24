//! Per-frame update, render dispatch, and UI overlay synchronization for `World`.

use super::{World, SPHERE_SIZE};
use crate::app::applied_config::AppliedConfig;
use crate::toolbox::camera::Camera;
use crate::toolbox::color::WHITE;
use crate::toolbox::input::Input;
use crate::toolbox::opengl::display_manager::DisplayManager;

impl World {
    /// Advances the world by one frame.
    ///
    /// The shared UI mutex is only held long enough to copy live scalar settings and detect a new
    /// committed `apply_counter`. If a new configuration is pending while tangent mode is active,
    /// the state is stashed in `deferred_apply_state` and applied only after tangent mode is
    /// forced back to world space. This avoids rebuilding geometry or field caches while the dive
    /// transition still depends on the old anchor and camera endpoints.
    ///
    /// Outside of that lock boundary, this method owns the full frame update: applying pending
    /// state, advancing tangent-space logic, invalidating render buffers when needed, and pushing
    /// overlay metadata back to the UI.
    pub fn update(
        &mut self,
        input: &Input,
        dt: f64,
        display_manager: &DisplayManager,
        camera: &mut Camera,
    ) {
        let render_state_before = self.tangent_space.render_state();
        let mut needs_render_rebuild = false;

        if let Some(state) = self.take_pending_apply_state(camera) {
            let next_config = AppliedConfig::from_ui(&state);
            let diff = self.applied_config.diff(&next_config);
            self.apply_state(state, next_config, diff);
            needs_render_rebuild = true;
        }

        self.tangent_space.update(
            input,
            dt,
            camera,
            display_manager,
            &self.grid_world,
            self.grid.get_coords(),
            self.renderer.projection,
        );
        //self.renderer.set_zoom_mix(self.tangent_space.scene_mix()); comment for now do not remove it!!
        if needs_render_rebuild || self.tangent_space.render_state() != render_state_before {
            self.rebuild_render_field();
        }
        self.sync_overlay_state();
        self.update_sphere();
    }

    /// Renders the current grid, field, tangent overlays, and marker sphere.
    ///
    /// Visibility of each layer is delegated to the tangent-space subsystem so world and
    /// tangent views stay synchronized.
    pub fn render(&self, camera: &Camera) {
        self.renderer.render(
            &self.grid,
            &self.render_field,
            &self.render_form_samples,
            self.tangent_space.show_grid(),
            self.show_vector_field(),
            camera,
            &self.sphere,
            &self.tangent_space.scene_transform(),
        )
    }

    fn take_pending_apply_state(
        &mut self,
        camera: &mut Camera,
    ) -> Option<crate::app::ui::GridUiState> {
        let mut pending_state = self.deferred_apply_state.take();
        {
            let shared = self.shared_ui_state.lock().unwrap();
            self.tangent_space
                .set_geometric_local_scale(shared.tangent_scale);
            self.tangent_space
                .set_geometric_arrow_scale(shared.geometric_arrow_scale);
            if pending_state.is_none() && self.last_counter != shared.apply_counter {
                pending_state = Some(shared.clone());
            }
        }

        if pending_state.is_some() && self.tangent_space.should_defer_apply() {
            self.deferred_apply_state = pending_state;
            self.tangent_space.force_world_mode(camera);
            return None;
        }

        pending_state
    }

    /// Refreshes the marker sphere that highlights the hovered or anchored sample.
    ///
    /// The marker is derived from tangent-space state each frame instead of being shared through
    /// the UI mutex, so there is no cross-thread ownership of renderable scene objects.
    fn update_sphere(&mut self) {
        if let Some(position) = self.tangent_space.marker_position() {
            self.sphere = Some(crate::graphics::model::Sphere::new(
                position,
                WHITE,
                SPHERE_SIZE,
            ));
        } else {
            self.sphere = None;
        }
    }

    /// Publishes overlay metadata back to the shared UI state.
    ///
    /// The shared lock is taken only for the scalar legend payload; renderables themselves remain
    /// owned by the main thread. This keeps the UI thread informed without turning the mutex into
    /// a transport for large scene structures.
    fn sync_overlay_state(&self) {
        let mut shared = self.shared_ui_state.lock().unwrap();
        shared.legend = self.legend;
    }
}
