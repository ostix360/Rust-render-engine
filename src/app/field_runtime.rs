//! Runtime field construction for direct scalar/vector input and derived `d(field)` renders.

use crate::app::grid::Grid;
use crate::app::ui::{GridUiState, SpacialEqs};
use crate::maths::differential::Form;
use crate::maths::field::{ScalarField, VectorField};

#[derive(Clone)]
pub enum RuntimeField {
    Scalar(ScalarField),
    Vector(VectorField),
}

impl RuntimeField {
    /// Builds the active runtime field from the committed UI state and current grid space.
    pub fn from_ui(state: &GridUiState, grid: &Grid) -> Self {
        let space = grid.get_coords().get_space();

        match (state.field_kind, state.render_d) {
            (crate::app::ui::FieldKind::Scalar, false) => {
                RuntimeField::Scalar(ScalarField::new(state.scalar_field.eq.clone()))
            }
            (crate::app::ui::FieldKind::Scalar, true) => RuntimeField::Vector(
                VectorField::gradient_from_scalar(state.scalar_field.eq.clone(), space),
            ),
            (crate::app::ui::FieldKind::Vector, false) => {
                RuntimeField::Vector(build_vector_field(&state.field, grid))
            }
            (crate::app::ui::FieldKind::Vector, true) => {
                let field_eqs = vec![
                    state.field.x.eq.clone(),
                    state.field.y.eq.clone(),
                    state.field.z.eq.clone(),
                ];
                RuntimeField::Vector(VectorField::curl_from_otn(
                    Form::new_otn(field_eqs, 1),
                    space,
                ))
            }
        }
    }

    /// Returns whether the active field render path produces arrows.
    pub fn is_vector_like(&self) -> bool {
        matches!(self, RuntimeField::Vector(_))
    }

    /// Returns the active vector field when present.
    pub fn as_vector(&self) -> Option<&VectorField> {
        match self {
            RuntimeField::Vector(field) => Some(field),
            RuntimeField::Scalar(_) => None,
        }
    }
}

/// Builds the runtime vector field from UI equations in the current active coordinates.
fn build_vector_field(field: &SpacialEqs, grid: &Grid) -> VectorField {
    let field_eqs = vec![field.x.eq.clone(), field.y.eq.clone(), field.z.eq.clone()];
    VectorField::from_otn(Form::new_otn(field_eqs, 1), grid.get_coords().get_space())
}
