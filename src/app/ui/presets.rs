//! Built-in UI presets for common coordinate systems and field configurations.

use crate::app::ui::state::{EmMode, FieldKind, GridUiState, SpacialEqs};

#[derive(Debug, Clone, Copy)]
pub(crate) struct GridPreset {
    pub(crate) label: &'static str,
    render_3d: bool,
    equations: [&'static str; 3],
    counts: [f64; 3],
    bounds: [(f64, f64); 3],
}

impl GridPreset {
    pub(crate) const ALL: [Self; 4] = [
        Self {
            label: "Cartesian",
            render_3d: true,
            equations: ["x", "y", "z"],
            counts: [7.0, 7.0, 7.0],
            bounds: [(-4.0, 4.0), (-4.0, 4.0), (-4.0, 4.0)],
        },
        Self {
            label: "Spherical",
            render_3d: true,
            equations: ["x*cos(y) * sin(z)", "x*sin(y) * sin(z)", "x * cos(z)"],
            counts: [5.0, 12.0, 8.0],
            bounds: [(0.0, 6.0), (0.0, 6.28), (0.0, 3.14)],
        },
        Self {
            label: "Cylindrical",
            render_3d: true,
            equations: ["x*cos(y)", "x*sin(y)", "z"],
            counts: [5.0, 12.0, 7.0],
            bounds: [(0.0, 6.0), (0.0, 6.28), (-4.0, 4.0)],
        },
        Self {
            label: "Polar",
            render_3d: false,
            equations: ["x*cos(y)", "x*sin(y)", "0"],
            counts: [6.0, 16.0, 2.0],
            bounds: [(0.0, 6.0), (0.0, 6.28), (0.0, 1.0)],
        },
    ];

    pub(crate) fn apply(self, state: &mut GridUiState) {
        state.render_3d = self.render_3d;
        state.coords_sys.x.eq_str = self.equations[0].to_string();
        state.coords_sys.y.eq_str = self.equations[1].to_string();
        state.coords_sys.z.eq_str = self.equations[2].to_string();
        state.nb_x = self.counts[0];
        state.nb_y = self.counts[1];
        state.nb_z = self.counts[2];
        state.bounds_x = self.bounds[0];
        state.bounds_y = self.bounds[1];
        state.bounds_z = self.bounds[2];
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FieldPreset {
    pub(crate) label: &'static str,
    kind: FieldKind,
    scalar: &'static str,
    vector: [&'static str; 3],
    render_d: bool,
    normalize: bool,
}

impl FieldPreset {
    pub(crate) const ALL: [Self; 4] = [
        Self {
            label: "Constant scalar",
            kind: FieldKind::Scalar,
            scalar: "1",
            vector: ["1", "0", "0"],
            render_d: false,
            normalize: false,
        },
        Self {
            label: "Variable scalar",
            kind: FieldKind::Scalar,
            scalar: "x*y + z",
            vector: ["1", "0", "0"],
            render_d: false,
            normalize: false,
        },
        Self {
            label: "Constant vector",
            kind: FieldKind::Vector,
            scalar: "x",
            vector: ["1", "0", "0"],
            render_d: false,
            normalize: false,
        },
        Self {
            label: "Variable vector",
            kind: FieldKind::Vector,
            scalar: "x",
            vector: ["-y", "x", "0.5*z"],
            render_d: false,
            normalize: false,
        },
    ];

    pub(crate) fn apply(self, state: &mut GridUiState) {
        state.field_kind = self.kind;
        state.scalar_field.eq_str = self.scalar.to_string();
        state.field.x.eq_str = self.vector[0].to_string();
        state.field.y.eq_str = self.vector[1].to_string();
        state.field.z.eq_str = self.vector[2].to_string();
        state.render_d = self.render_d;
        state.normalize_field = self.normalize;
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EmPreset {
    pub(crate) label: &'static str,
    phi: &'static str,
    vector_potential: [&'static str; 3],
    electric_field: [&'static str; 3],
    magnetic_field: [&'static str; 3],
}

impl EmPreset {
    pub(crate) const ALL: [Self; 3] = [
        Self {
            label: "Plane wave",
            phi: "0",
            vector_potential: ["0", "sin(z - t)", "0"],
            electric_field: ["0", "cos(z - t)", "0"],
            magnetic_field: ["-cos(z - t)", "0", "0"],
        },
        Self {
            label: "Standing wave",
            phi: "0",
            vector_potential: ["0", "sin(z - t) + sin(z + t)", "0"],
            electric_field: ["0", "cos(z - t) - cos(z + t)", "0"],
            magnetic_field: ["-cos(z - t) - cos(z + t)", "0", "0"],
        },
        Self {
            label: "Damped wave",
            phi: "0",
            vector_potential: ["0", "exp(-0.25*z) * sin(z - t)", "0"],
            electric_field: ["0", "exp(-0.25*z) * cos(z - t)", "0"],
            magnetic_field: ["exp(-0.25*z) * (0.25*sin(z - t) - cos(z - t))", "0", "0"],
        },
    ];

    pub(crate) fn apply(self, state: &mut GridUiState) {
        state.em.mode = EmMode::Potentials;
        state.em.phi.eq_str = self.phi.to_string();
        set_spacial_eqs(&mut state.em.vector_potential, self.vector_potential);
        set_spacial_eqs(&mut state.em.electric_field, self.electric_field);
        set_spacial_eqs(&mut state.em.magnetic_field, self.magnetic_field);
        state.em.layers.scalar_potential = false;
        state.em.layers.vector_potential = false;
        state.em.layers.electric = true;
        state.em.layers.magnetic = true;
    }
}

fn set_spacial_eqs(eqs: &mut SpacialEqs, values: [&str; 3]) {
    eqs.x.eq_str = values[0].to_string();
    eqs.y.eq_str = values[1].to_string();
    eqs.z.eq_str = values[2].to_string();
}

#[cfg(test)]
mod tests {
    use super::{EmPreset, FieldPreset, GridPreset};
    use crate::app::ui::state::GridUiState;
    use crate::app::ui::validation::validate_ui_state;

    #[test]
    fn grid_presets_are_valid_after_apply() {
        for preset in GridPreset::ALL {
            let mut state = GridUiState::default();
            preset.apply(&mut state);

            let result = validate_ui_state(&state);

            assert!(result.is_ok(), "{} preset should validate", preset.label);
        }
    }

    #[test]
    fn field_presets_are_valid_after_apply() {
        for preset in FieldPreset::ALL {
            let mut state = GridUiState::default();
            preset.apply(&mut state);

            let result = validate_ui_state(&state);

            assert!(result.is_ok(), "{} preset should validate", preset.label);
        }
    }

    #[test]
    fn em_presets_are_valid_and_hide_potential_layers_after_apply() {
        for preset in EmPreset::ALL {
            let mut state = GridUiState::default();
            state.em.enabled = true;
            state.em.layers.scalar_potential = true;
            state.em.layers.vector_potential = true;
            state.em.layers.electric = false;
            state.em.layers.magnetic = false;
            preset.apply(&mut state);

            let result = validate_ui_state(&state);

            assert!(result.is_ok(), "{} preset should validate", preset.label);
            assert!(!state.em.layers.scalar_potential);
            assert!(!state.em.layers.vector_potential);
            assert!(state.em.layers.electric);
            assert!(state.em.layers.magnetic);
        }
    }
}
