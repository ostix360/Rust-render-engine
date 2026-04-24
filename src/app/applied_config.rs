//! Applied UI configuration snapshots and diffing.

use crate::app::grid::GridConfig;
use crate::app::ui::{FieldKind, GridUiState};
use mathhook_core::formatter::simple::SimpleContext;
use mathhook_core::SimpleFormatter;

#[derive(Clone, PartialEq)]
pub(crate) struct AppliedConfig {
    pub(crate) grid_config: GridConfig,
    pub(crate) coord_eqs: [String; 3],
    field_kind: FieldKind,
    scalar_eq: String,
    vector_eqs: [String; 3],
    render_d: bool,
    pub(crate) normalize_field: bool,
}

impl AppliedConfig {
    /// Builds the subset of UI state that drives world reconfiguration.
    pub(crate) fn from_ui(state: &GridUiState) -> Self {
        let context = SimpleContext::default();
        Self {
            grid_config: state.to_grid_config(),
            coord_eqs: [
                state
                    .coords_sys
                    .x
                    .eq
                    .to_simple(&context)
                    .expect("Error while converting x eq"),
                state
                    .coords_sys
                    .y
                    .eq
                    .to_simple(&context)
                    .expect("Error while converting y eq"),
                state
                    .coords_sys
                    .z
                    .eq
                    .to_simple(&context)
                    .expect("Error while converting z eq"),
            ],
            field_kind: state.field_kind,
            scalar_eq: state.scalar_field.eq_str.clone(),
            vector_eqs: [
                state.field.x.eq_str.clone(),
                state.field.y.eq_str.clone(),
                state.field.z.eq_str.clone(),
            ],
            render_d: state.render_d,
            normalize_field: state.normalize_field,
        }
    }

    /// Computes which high-level parts of the world changed between two snapshots.
    pub(crate) fn diff(&self, next: &Self) -> ApplyDiff {
        ApplyDiff {
            grid_changed: self.grid_config != next.grid_config,
            coords_changed: self.coord_eqs != next.coord_eqs,
            field_kind_changed: self.field_kind != next.field_kind,
            scalar_changed: self.scalar_eq != next.scalar_eq,
            vector_changed: self.vector_eqs != next.vector_eqs,
            render_d_changed: self.render_d != next.render_d,
            normalize_changed: self.normalize_field != next.normalize_field,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ApplyDiff {
    pub(crate) grid_changed: bool,
    pub(crate) coords_changed: bool,
    pub(crate) field_kind_changed: bool,
    pub(crate) scalar_changed: bool,
    pub(crate) vector_changed: bool,
    pub(crate) render_d_changed: bool,
    pub(crate) normalize_changed: bool,
}

impl ApplyDiff {
    /// Returns whether the grid geometry or coordinate embedding changed.
    pub(crate) fn geometry_changed(self) -> bool {
        self.grid_changed || self.coords_changed
    }

    /// Returns whether the active runtime field must be rebuilt.
    pub(crate) fn runtime_field_changed(self) -> bool {
        self.coords_changed
            || self.field_kind_changed
            || self.scalar_changed
            || self.vector_changed
            || self.render_d_changed
    }

    /// Returns whether cached field samples must be recomputed.
    pub(crate) fn field_cache_changed(self) -> bool {
        self.geometry_changed() || self.runtime_field_changed()
    }
}
