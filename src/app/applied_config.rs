//! Applied UI configuration snapshots and diffing.

use crate::app::grid::GridConfig;
use crate::app::ui::{EmGauge, EmLayerVisibility, EmMode, FieldKind, GridUiState};
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
    em_enabled: bool,
    em_mode: EmMode,
    em_gauge: EmGauge,
    em_phi: String,
    em_a_eqs: [String; 3],
    em_e_eqs: [String; 3],
    em_b_eqs: [String; 3],
    em_normalize_vectors: bool,
    em_light_speed_bits: u64,
    em_magnetic_vector_scale_bits: u64,
    em_layers: EmLayerVisibility,
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
            em_enabled: state.em.enabled,
            em_mode: state.em.mode,
            em_gauge: state.em.gauge,
            em_phi: state.em.phi.eq_str.clone(),
            em_a_eqs: [
                state.em.vector_potential.x.eq_str.clone(),
                state.em.vector_potential.y.eq_str.clone(),
                state.em.vector_potential.z.eq_str.clone(),
            ],
            em_e_eqs: [
                state.em.electric_field.x.eq_str.clone(),
                state.em.electric_field.y.eq_str.clone(),
                state.em.electric_field.z.eq_str.clone(),
            ],
            em_b_eqs: [
                state.em.magnetic_field.x.eq_str.clone(),
                state.em.magnetic_field.y.eq_str.clone(),
                state.em.magnetic_field.z.eq_str.clone(),
            ],
            em_normalize_vectors: state.em.normalize_vectors,
            em_light_speed_bits: state.em.light_speed.to_bits(),
            em_magnetic_vector_scale_bits: state.em.magnetic_vector_scale.to_bits(),
            em_layers: state.em.layers.clone(),
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
            em_enabled_changed: self.em_enabled != next.em_enabled,
            em_mode_changed: self.em_mode != next.em_mode,
            em_equations_changed: self.em_gauge != next.em_gauge
                || self.em_phi != next.em_phi
                || self.em_a_eqs != next.em_a_eqs
                || self.em_e_eqs != next.em_e_eqs
                || self.em_b_eqs != next.em_b_eqs
                || self.em_light_speed_bits != next.em_light_speed_bits
                || self.em_magnetic_vector_scale_bits != next.em_magnetic_vector_scale_bits,
            em_normalize_changed: self.em_normalize_vectors != next.em_normalize_vectors,
            em_layers_changed: self.em_layers != next.em_layers,
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
    pub(crate) em_enabled_changed: bool,
    pub(crate) em_mode_changed: bool,
    pub(crate) em_equations_changed: bool,
    pub(crate) em_normalize_changed: bool,
    pub(crate) em_layers_changed: bool,
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

    pub(crate) fn em_runtime_changed(self) -> bool {
        self.coords_changed
            || self.grid_changed
            || self.em_enabled_changed
            || self.em_mode_changed
            || self.em_equations_changed
    }

    pub(crate) fn em_render_changed(self) -> bool {
        self.geometry_changed()
            || self.em_runtime_changed()
            || self.em_normalize_changed
            || self.em_layers_changed
    }

    /// Returns whether cached field samples must be recomputed.
    pub(crate) fn field_cache_changed(self) -> bool {
        self.geometry_changed() || self.runtime_field_changed()
    }
}
