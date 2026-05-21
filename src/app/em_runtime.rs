//! Electromagnetism runtime fields over animated 3D slices.

mod fields;
mod maxwell;
mod plane_wave;
mod potentials;

#[cfg(test)]
mod cache_tests;
#[cfg(test)]
mod plane_wave_tests;
#[cfg(test)]
mod runtime_tests;

use crate::app::grid::{Grid, GridConfig};
use crate::app::ui::{EmLayerVisibility, EmMode, EmUiState};
use crate::maths::differential::Form;
use crate::maths::space::Space;
use crate::maths::{derivate, Expr, ExternalDerivative, Point};
use fields::{TimedScalarField, TimedVectorField};
use mathhook_core::Simplify;
use maxwell::{
    maxwell_ampere_source_exprs, maxwell_faraday_source_exprs, maxwell_inverse_curl,
    MaxwellSolveConfig,
};
use nalgebra::Vector3;
use plane_wave::{plane_wave_electric_exprs, plane_wave_magnetic_exprs};
use potentials::{local_vector_potential_from_b, scalar_potential_for_gauge};
use std::ops::{Add, Mul};

pub struct EmRuntime {
    pub layers: EmLayerVisibility,
    magnetic_vector_scale: f64,
    phi: TimedScalarField,
    vector_potential: TimedVectorField,
    electric_field: TimedVectorField,
    magnetic_field: TimedVectorField,
}

impl EmRuntime {
    #[allow(dead_code)]
    pub fn from_ui(state: &EmUiState, grid: &Grid) -> Self {
        Self::from_ui_with_config(state, grid, GridConfig::default())
    }

    pub fn from_ui_with_config(state: &EmUiState, grid: &Grid, grid_config: GridConfig) -> Self {
        let maxwell_config =
            MaxwellSolveConfig::from_grid_config(grid_config, grid.get_coords().sample_geometry());
        match state.mode {
            EmMode::Potentials => Self::from_potentials(state, grid.get_coords().get_space()),
            EmMode::Electric => Self::from_electric(state, maxwell_config),
            EmMode::Magnetic => Self::from_magnetic(state, maxwell_config),
        }
    }

    fn from_potentials(state: &EmUiState, space: &Space) -> Self {
        let phi_expr = state.phi.eq.clone();
        let a_otn_exprs = exprs_from_spacial(&state.vector_potential);
        let a_otn = Form::new_otn(a_otn_exprs.to_vec(), 1);
        let a_natural = a_otn.to_dual_base(space);

        let mut phi_form = Form::new(vec![phi_expr.clone()], 0);
        let grad_phi = phi_form.d();
        let electric_natural = Form::new(
            (0..3)
                .map(|index| {
                    negate(grad_phi.get_expr(index).clone())
                        .add(negate(partial_t(a_natural.get_expr(index).clone())))
                        .simplify()
                })
                .collect(),
            1,
        );
        let electric_otn = electric_natural.to_otn_base(space);
        let mut a_for_d = a_natural;
        let magnetic_otn = a_for_d.d().to_otn_base(space).hodge_star_otn_3d();

        Self {
            layers: state.layers.clone(),
            magnetic_vector_scale: state.magnetic_vector_scale,
            phi: TimedScalarField::new(phi_expr),
            vector_potential: TimedVectorField::from_exprs(a_otn_exprs),
            electric_field: TimedVectorField::from_exprs(form_exprs(&electric_otn)),
            magnetic_field: TimedVectorField::from_exprs(form_exprs(&magnetic_otn)),
        }
    }

    fn from_electric(state: &EmUiState, maxwell_config: MaxwellSolveConfig) -> Self {
        let electric_exprs = exprs_from_spacial(&state.electric_field);
        let c = state.light_speed.max(1.0e-6);

        let electric_field = TimedVectorField::from_exprs(electric_exprs.clone());
        let magnetic_field =
            if let Some(magnetic_exprs) = plane_wave_magnetic_exprs(&electric_exprs, c) {
                TimedVectorField::from_exprs(magnetic_exprs)
            } else {
                let ampere_source_exprs = maxwell_ampere_source_exprs(&electric_exprs, c);
                let ampere_source = TimedVectorField::from_exprs(ampere_source_exprs);
                maxwell_inverse_curl(ampere_source, maxwell_config)
            };
        let vector_potential = local_vector_potential_from_b(magnetic_field.clone());
        let phi = scalar_potential_for_gauge(state.gauge, electric_field.clone());

        Self {
            layers: state.layers.clone(),
            magnetic_vector_scale: state.magnetic_vector_scale,
            phi,
            vector_potential,
            electric_field,
            magnetic_field,
        }
    }

    fn from_magnetic(state: &EmUiState, maxwell_config: MaxwellSolveConfig) -> Self {
        let magnetic_exprs = exprs_from_spacial(&state.magnetic_field);
        let c = state.light_speed.max(1.0e-6);

        let magnetic_field = TimedVectorField::from_exprs(magnetic_exprs.clone());
        let electric_field =
            if let Some(electric_exprs) = plane_wave_electric_exprs(&magnetic_exprs, c) {
                TimedVectorField::from_exprs(electric_exprs)
            } else {
                let faraday_source_exprs = maxwell_faraday_source_exprs(&magnetic_exprs);
                let faraday_source = TimedVectorField::from_exprs(faraday_source_exprs);
                maxwell_inverse_curl(faraday_source, maxwell_config.clone())
            };
        let phi = scalar_potential_for_gauge(state.gauge, electric_field.clone());
        let vector_potential = maxwell_inverse_curl(magnetic_field.clone(), maxwell_config);

        Self {
            layers: state.layers.clone(),
            magnetic_vector_scale: state.magnetic_vector_scale,
            phi,
            vector_potential,
            electric_field,
            magnetic_field,
        }
    }

    pub fn phi_at(&self, point: Point, time: f64) -> f64 {
        self.phi.at(point, time)
    }

    pub fn vector_potential_at(&self, point: Point, time: f64) -> Vector3<f64> {
        self.vector_potential.at(point, time)
    }

    pub fn electric_at(&self, point: Point, time: f64) -> Vector3<f64> {
        self.electric_field.at(point, time)
    }

    pub fn magnetic_at(&self, point: Point, time: f64) -> Vector3<f64> {
        self.magnetic_field.at(point, time)
    }

    pub fn magnetic_render_scale(&self) -> f64 {
        self.magnetic_vector_scale
    }

    pub fn active_layers(&self) -> EmLayerVisibility {
        self.layers.clone()
    }

    pub fn active_vector_layer_count(&self) -> usize {
        let layers = self.active_layers();
        usize::from(layers.electric)
            + usize::from(layers.magnetic)
            + usize::from(layers.vector_potential)
    }

    pub fn prewarm_vector_layer_times(
        &self,
        point: Point,
        times: &[f64],
        layers: &EmLayerVisibility,
    ) {
        for &time in times {
            if layers.electric {
                let _ = self.electric_at(point, time);
            }
            if layers.magnetic {
                let _ = self.magnetic_at(point, time);
            }
            if layers.vector_potential {
                let _ = self.vector_potential_at(point, time);
            }
        }
    }
}

fn exprs_from_spacial(eqs: &crate::app::ui::SpacialEqs) -> [Expr; 3] {
    [eqs.x.eq.clone(), eqs.y.eq.clone(), eqs.z.eq.clone()]
}

fn form_exprs(form: &Form) -> [Expr; 3] {
    [
        form.get_expr(0).clone(),
        form.get_expr(1).clone(),
        form.get_expr(2).clone(),
    ]
}

fn partial_t(expr: Expr) -> Expr {
    derivate(expr, &"t".to_string())
}

fn negate(expr: Expr) -> Expr {
    Expr::number(-1.0).mul(expr)
}
