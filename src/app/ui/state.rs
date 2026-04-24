//! Shared UI state exchanged between the control window and the render loop.

use crate::app::grid::GridConfig;
use crate::maths::Expr;
use mathhook_core::Parser;
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct EqRender {
    pub eq: Expr,
    pub eq_str: String,
}

impl EqRender {
    /// Creates one parsed equation together with its editable source string.
    pub fn new(eq: Expr, eq_str: String) -> Self {
        Self { eq, eq_str }
    }
}

#[derive(Debug, Clone)]
pub struct SpacialEqs {
    pub x: EqRender,
    pub y: EqRender,
    pub z: EqRender,
}

impl SpacialEqs {
    /// Constructs `SpacialEqs` from defaults.
    ///
    /// It is part of the egui control window and keeps UI state in sync with the shared runtime
    /// state.
    pub(crate) fn from_defaults(x: &str, y: &str, z: &str) -> Self {
        Self {
            x: default_eq(x),
            y: default_eq(y),
            z: default_eq(z),
        }
    }

    /// Builds the default spherical-style coordinate-system equations.
    pub fn default_sys() -> Self {
        Self::from_defaults("x*cos(y) * sin(z)", "x*sin(y) * sin(z)", "x * cos(z)")
    }

    /// Builds the default constant vector-field equations.
    pub fn default_field() -> Self {
        Self::from_defaults("1", "0", "0")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Scalar,
    Vector,
}

#[derive(Debug, Clone)]
pub struct GridUiState {
    pub render_3d: bool,
    pub coords_sys: SpacialEqs,
    pub field_kind: FieldKind,
    pub scalar_field: EqRender,
    pub field: SpacialEqs,
    pub render_d: bool,
    pub normalize_field: bool,
    pub tangent_scale: f64,
    pub geometric_arrow_scale: f64,
    pub nb_x: f64,
    pub nb_y: f64,
    pub nb_z: f64,
    pub bounds_x: (f64, f64),
    pub bounds_y: (f64, f64),
    pub bounds_z: (f64, f64),
    pub apply_counter: u64,
    pub legend: Option<LegendState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendKind {
    ScalarField,
    DualTangent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegendDescriptor {
    pub window_title: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub footer: &'static str,
}

impl LegendKind {
    /// Returns the static UI copy associated with this legend source.
    pub fn descriptor(self) -> LegendDescriptor {
        match self {
            Self::ScalarField => LegendDescriptor {
                window_title: "Scalar Field Legend",
                title: "Scalar Field Legend",
                subtitle: "Sampled field values over the current grid",
                footer: "Visible when rendering the base scalar field.",
            },
            Self::DualTangent => LegendDescriptor {
                window_title: "Dual Tangent Legend",
                title: "Dual Tangent Legend",
                subtitle: "alpha(v) over the sampled dual-space lattice",
                footer: "Visible only in dual tangent mode: Ctrl+T.",
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegendState {
    pub kind: LegendKind,
    pub min_value: f64,
    pub max_value: f64,
}

impl GridUiState {
    /// Rounds and normalizes the editable UI bounds into a runtime `GridConfig`.
    ///
    /// The control panel stores editable floating-point values because sliders and text fields
    /// operate in that domain. The runtime grid builder, however, expects stable snapped values
    /// so cache keys and segment generation do not drift across frames. The special handling for
    /// `3.14` and `6.28` preserves the common `PI` and `2*PI` intent from the UI.
    pub fn approximate_grid_config(&self) -> GridConfig {
        let normalize_bound = |value: f64| match value {
            value if value == 3.14 => PI,
            value if value == 6.28 => 2.0 * PI,
            _ => value.round(),
        };

        GridConfig::new(
            normalize_bound(self.bounds_x.0),
            normalize_bound(self.bounds_x.1),
            self.nb_x.round(),
            normalize_bound(self.bounds_y.0),
            normalize_bound(self.bounds_y.1),
            self.nb_y.round(),
            normalize_bound(self.bounds_z.0),
            normalize_bound(self.bounds_z.1),
            self.nb_z.round(),
        )
    }

    /// Converts the current UI state into the grid configuration used by the runtime.
    ///
    /// This is the configuration snapshot compared by `World` when deciding whether geometry and
    /// lookup caches must be rebuilt.
    pub fn to_grid_config(&self) -> GridConfig {
        self.approximate_grid_config()
    }

    /// Returns whether the active field render path should draw arrows.
    pub fn renders_vector_field(&self) -> bool {
        self.field_kind == FieldKind::Vector || self.render_d
    }

    /// Returns whether the active field render path should draw sampled scalar spheres.
    pub fn renders_scalar_samples(&self) -> bool {
        self.field_kind == FieldKind::Scalar && !self.render_d
    }
}

impl Default for GridUiState {
    /// Builds the default `GridUiState`.
    ///
    /// The defaults are chosen so the first frame can immediately build a valid grid and vector
    /// field without any UI interaction. `apply_counter` starts at zero and is only bumped after
    /// validation succeeds, which lets the render thread cheaply detect committed edits without
    /// reacting to every intermediate keystroke.
    fn default() -> Self {
        Self {
            render_3d: true,
            coords_sys: SpacialEqs::default_sys(),
            field_kind: FieldKind::Vector,
            scalar_field: default_eq("x"),
            field: SpacialEqs::default_field(),
            render_d: false,
            normalize_field: false,
            tangent_scale: 0.12,
            geometric_arrow_scale: 0.55,
            nb_x: 5.0,
            nb_y: 5.0,
            nb_z: 5.0,
            bounds_x: (-0.1, 15.0),
            bounds_y: (0.0, 6.28),
            bounds_z: (0.0, 3.14),
            apply_counter: 0,
            legend: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControlTab {
    Grid,
    Field,
}

/// Parses one default equation string into `EqRender`.
///
/// Defaults are expected to be valid and panic if they are not, because invalid built-in
/// equations would make the application fail at startup.
fn default_eq(expr: &str) -> EqRender {
    EqRender::new(Parser::default().parse(expr).unwrap(), expr.to_string())
}

#[cfg(test)]
mod tests {
    use super::{ControlTab, FieldKind, GridUiState};

    #[test]
    fn grid_ui_state_defaults_match_expected_values() {
        let state = GridUiState::default();

        assert!(state.render_3d);
        assert_eq!(state.field_kind, FieldKind::Vector);
        assert!(!state.render_d);
        assert!(!state.normalize_field);
        assert_eq!(state.tangent_scale, 0.12);
        assert_eq!(state.geometric_arrow_scale, 0.55);
        assert_eq!(state.nb_x, 5.0);
        assert_eq!(state.nb_y, 5.0);
        assert_eq!(state.nb_z, 5.0);
        assert_eq!(state.bounds_x, (-0.1, 15.0));
        assert_eq!(state.bounds_y, (0.0, 6.28));
        assert_eq!(state.bounds_z, (0.0, 3.14));
        assert_eq!(state.legend, None);
    }

    #[test]
    fn control_tab_defaults_to_grid_in_callers() {
        let tab = ControlTab::Grid;
        assert!(matches!(tab, ControlTab::Grid));
    }

    #[test]
    fn scalar_mode_without_d_renders_samples() {
        let mut state = GridUiState::default();
        state.field_kind = FieldKind::Scalar;

        assert!(state.renders_scalar_samples());
        assert!(!state.renders_vector_field());
    }

    #[test]
    fn scalar_mode_with_d_renders_vectors() {
        let mut state = GridUiState::default();
        state.field_kind = FieldKind::Scalar;
        state.render_d = true;

        assert!(!state.renders_scalar_samples());
        assert!(state.renders_vector_field());
    }
}
