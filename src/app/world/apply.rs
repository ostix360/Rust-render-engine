//! UI configuration application and cache invalidation for `World`.

use super::World;
use crate::app::applied_config::{AppliedConfig, ApplyDiff};
use crate::app::coords_sys::CoordsSys;
use crate::app::field_runtime::RuntimeField;
use crate::app::ui::GridUiState;

impl World {
    /// Applies validated UI state to the world and refreshes whichever caches changed.
    ///
    /// The caller is expected to pass in a fully cloned and validated `GridUiState`, so this
    /// method does not acquire the shared UI lock itself. That keeps the critical section short:
    /// lock on the UI thread, clone, unlock, then do the potentially expensive grid, kd-tree,
    /// shader, and field-cache rebuilds here on the render thread.
    pub(super) fn apply_state(
        &mut self,
        state: GridUiState,
        next_config: AppliedConfig,
        diff: ApplyDiff,
    ) {
        self.apply_coordinate_changes(&state, &next_config, diff);
        self.apply_field_changes(&state, &next_config, diff);
        self.applied_config = next_config;
        self.last_counter = state.apply_counter;
    }

    fn apply_coordinate_changes(
        &mut self,
        state: &GridUiState,
        next_config: &AppliedConfig,
        diff: ApplyDiff,
    ) {
        if diff.coords_changed {
            let coords = state.coords_sys.clone();
            let coord_sys = CoordsSys::new(coords.x.eq, coords.y.eq, coords.z.eq);
            self.grid.set_coordinates(coord_sys);
            self.renderer
                .grid_renderer
                .update_shader_eqs(&next_config.coord_eqs);
        }

        if diff.geometry_changed() {
            self.grid.update_config(&next_config.grid_config);
            let (field_samples, grid_samples) = Self::build_grid_cache(&self.grid);
            self.field_samples = field_samples;
            self.grid_world.replace_samples(grid_samples);
        }
    }

    fn apply_field_changes(
        &mut self,
        state: &GridUiState,
        next_config: &AppliedConfig,
        diff: ApplyDiff,
    ) {
        if diff.runtime_field_changed() {
            self.field = RuntimeField::from_ui(state, &self.grid);
        }

        self.normalize_field = next_config.normalize_field;

        if diff.field_cache_changed() {
            self.recompute_cached_field_data();
        }
    }
}
