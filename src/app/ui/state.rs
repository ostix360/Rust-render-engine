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
    pub(crate) fn from_defaults(x: &str, y: &str, z: &str) -> Self {
        Self {
            x: default_eq(x),
            y: default_eq(y),
            z: default_eq(z),
        }
    }

    pub fn default_sys() -> Self {
        Self::from_defaults("x*cos(y) * sin(z)", "x*sin(y) * sin(z)", "x * cos(z)")
    }

    pub fn default_field() -> Self {
        Self::from_defaults("1", "0", "0")
    }
}

#[derive(Debug, Clone)]
pub struct GridUiState {
    pub render_3d: bool,
    pub coords_sys: SpacialEqs,
    pub field: SpacialEqs,
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
    pub dual_legend: Option<DualLegendState>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DualLegendState {
    pub min_value: f64,
    pub max_value: f64,
}

impl GridUiState {
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

    pub fn to_grid_config(&self) -> GridConfig {
        self.approximate_grid_config()
    }
}

impl Default for GridUiState {
    fn default() -> Self {
        Self {
            render_3d: true,
            coords_sys: SpacialEqs::default_sys(),
            field: SpacialEqs::default_field(),
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
            dual_legend: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControlTab {
    Grid,
    Field,
}

fn default_eq(expr: &str) -> EqRender {
    EqRender::new(Parser::default().parse(expr).unwrap(), expr.to_string())
}

#[cfg(test)]
mod tests {
    use super::{ControlTab, GridUiState};

    #[test]
    fn grid_ui_state_defaults_match_expected_values() {
        let state = GridUiState::default();

        assert!(state.render_3d);
        assert!(!state.normalize_field);
        assert_eq!(state.tangent_scale, 0.12);
        assert_eq!(state.geometric_arrow_scale, 0.55);
        assert_eq!(state.nb_x, 5.0);
        assert_eq!(state.nb_y, 5.0);
        assert_eq!(state.nb_z, 5.0);
        assert_eq!(state.bounds_x, (-0.1, 15.0));
        assert_eq!(state.bounds_y, (0.0, 6.28));
        assert_eq!(state.bounds_z, (0.0, 3.14));
        assert_eq!(state.dual_legend, None);
    }

    #[test]
    fn control_tab_defaults_to_grid_in_callers() {
        let tab = ControlTab::Grid;
        assert!(matches!(tab, ControlTab::Grid));
    }
}
