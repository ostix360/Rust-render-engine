//! Parsing and validation helpers for equations entered in the control window.

use crate::app::ui::state::{EmMode, EmUiState, EqRender, FieldKind, GridUiState, SpacialEqs};
use mathhook_core::Parser;

#[derive(Debug)]
pub(crate) struct ValidatedUiState {
    pub coords_sys: SpacialEqs,
    pub scalar_field: EqRender,
    pub field: SpacialEqs,
    pub em: EmUiState,
}

/// Validates and reparses every editable equation in the UI state before apply.
///
/// Only equations that can affect the active render mode are reparsed. Inactive scalar/vector
/// fields keep their previous parsed expression so the user can switch modes without losing a
/// temporarily invalid draft in the hidden section.
pub(crate) fn validate_ui_state(state: &GridUiState) -> Result<ValidatedUiState, String> {
    let coord_x = validate_xyz_equation("Coordinate x", &state.coords_sys.x.eq_str);
    let coord_y = validate_xyz_equation("Coordinate y", &state.coords_sys.y.eq_str);
    let coord_z = validate_xyz_equation("Coordinate z", &state.coords_sys.z.eq_str);
    let scalar_field = match (state.em.enabled, state.field_kind) {
        (true, _) | (false, FieldKind::Vector) => Ok(state.scalar_field.clone()),
        (false, FieldKind::Scalar) => {
            validate_xyz_equation("Scalar field", &state.scalar_field.eq_str)
        }
    };
    let field_x = match (state.em.enabled, state.field_kind) {
        (true, _) | (false, FieldKind::Scalar) => Ok(state.field.x.clone()),
        (false, FieldKind::Vector) => validate_xyz_equation("Field Fx", &state.field.x.eq_str),
    };
    let field_y = match (state.em.enabled, state.field_kind) {
        (true, _) | (false, FieldKind::Scalar) => Ok(state.field.y.clone()),
        (false, FieldKind::Vector) => validate_xyz_equation("Field Fy", &state.field.y.eq_str),
    };
    let field_z = match (state.em.enabled, state.field_kind) {
        (true, _) | (false, FieldKind::Scalar) => Ok(state.field.z.clone()),
        (false, FieldKind::Vector) => validate_xyz_equation("Field Fz", &state.field.z.eq_str),
    };
    let em = validate_em_state(&state.em);

    let mut errors = Vec::new();
    collect_error(&coord_x, &mut errors);
    collect_error(&coord_y, &mut errors);
    collect_error(&coord_z, &mut errors);
    collect_error(&scalar_field, &mut errors);
    collect_error(&field_x, &mut errors);
    collect_error(&field_y, &mut errors);
    collect_error(&field_z, &mut errors);
    if let Err(error) = &em {
        errors.push(error.clone());
    }

    if !errors.is_empty() {
        return Err(format_error_summary(&errors));
    }

    Ok(ValidatedUiState {
        coords_sys: SpacialEqs {
            x: coord_x?,
            y: coord_y?,
            z: coord_z?,
        },
        scalar_field: scalar_field?,
        field: SpacialEqs {
            x: field_x?,
            y: field_y?,
            z: field_z?,
        },
        em: em?,
    })
}

/// Parses one equation string and rejects unsupported variables or empty input.
fn validate_xyz_equation(label: &str, eq: &str) -> Result<EqRender, String> {
    validate_equation(label, eq, &["x", "y", "z"])
}

fn validate_xyzt_equation(label: &str, eq: &str) -> Result<EqRender, String> {
    validate_equation(label, eq, &["x", "y", "z", "t"])
}

fn validate_equation(
    label: &str,
    eq: &str,
    allowed_variables: &[&str],
) -> Result<EqRender, String> {
    if eq.is_empty() {
        return Err(format!("{label}: Equation cannot be empty"));
    }

    let formal_eq = Parser::default()
        .parse(eq)
        .map_err(|error| format!("{label}: Invalid equation: {error}"))?;

    for variable in formal_eq.find_variables() {
        if !allowed_variables.contains(&variable.name()) {
            return Err(format!(
                "{label}: Invalid variable '{}'. Only {} are allowed.",
                variable.name(),
                format_allowed_variables(allowed_variables)
            ));
        }
    }

    Ok(EqRender::new(formal_eq, eq.to_string()))
}

fn validate_em_state(state: &EmUiState) -> Result<EmUiState, String> {
    if !state.enabled {
        return Ok(state.clone());
    }

    let phi = match state.mode {
        EmMode::Potentials => validate_xyzt_equation("EM phi", &state.phi.eq_str),
        EmMode::Electric | EmMode::Magnetic => Ok(state.phi.clone()),
    };
    let ax = match state.mode {
        EmMode::Potentials => validate_xyzt_equation("EM Ax", &state.vector_potential.x.eq_str),
        EmMode::Electric | EmMode::Magnetic => Ok(state.vector_potential.x.clone()),
    };
    let ay = match state.mode {
        EmMode::Potentials => validate_xyzt_equation("EM Ay", &state.vector_potential.y.eq_str),
        EmMode::Electric | EmMode::Magnetic => Ok(state.vector_potential.y.clone()),
    };
    let az = match state.mode {
        EmMode::Potentials => validate_xyzt_equation("EM Az", &state.vector_potential.z.eq_str),
        EmMode::Electric | EmMode::Magnetic => Ok(state.vector_potential.z.clone()),
    };
    let ex = match state.mode {
        EmMode::Electric => validate_xyzt_equation("EM Ex", &state.electric_field.x.eq_str),
        EmMode::Potentials | EmMode::Magnetic => Ok(state.electric_field.x.clone()),
    };
    let ey = match state.mode {
        EmMode::Electric => validate_xyzt_equation("EM Ey", &state.electric_field.y.eq_str),
        EmMode::Potentials | EmMode::Magnetic => Ok(state.electric_field.y.clone()),
    };
    let ez = match state.mode {
        EmMode::Electric => validate_xyzt_equation("EM Ez", &state.electric_field.z.eq_str),
        EmMode::Potentials | EmMode::Magnetic => Ok(state.electric_field.z.clone()),
    };
    let bx = match state.mode {
        EmMode::Magnetic => validate_xyzt_equation("EM Bx", &state.magnetic_field.x.eq_str),
        EmMode::Potentials | EmMode::Electric => Ok(state.magnetic_field.x.clone()),
    };
    let by = match state.mode {
        EmMode::Magnetic => validate_xyzt_equation("EM By", &state.magnetic_field.y.eq_str),
        EmMode::Potentials | EmMode::Electric => Ok(state.magnetic_field.y.clone()),
    };
    let bz = match state.mode {
        EmMode::Magnetic => validate_xyzt_equation("EM Bz", &state.magnetic_field.z.eq_str),
        EmMode::Potentials | EmMode::Electric => Ok(state.magnetic_field.z.clone()),
    };

    let mut errors = Vec::new();
    collect_error(&phi, &mut errors);
    collect_error(&ax, &mut errors);
    collect_error(&ay, &mut errors);
    collect_error(&az, &mut errors);
    collect_error(&ex, &mut errors);
    collect_error(&ey, &mut errors);
    collect_error(&ez, &mut errors);
    collect_error(&bx, &mut errors);
    collect_error(&by, &mut errors);
    collect_error(&bz, &mut errors);

    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    let mut validated = state.clone();
    validated.phi = phi?;
    validated.vector_potential = SpacialEqs {
        x: ax?,
        y: ay?,
        z: az?,
    };
    validated.electric_field = SpacialEqs {
        x: ex?,
        y: ey?,
        z: ez?,
    };
    validated.magnetic_field = SpacialEqs {
        x: bx?,
        y: by?,
        z: bz?,
    };
    Ok(validated)
}

fn format_allowed_variables(allowed_variables: &[&str]) -> String {
    match allowed_variables {
        ["x", "y", "z"] => "'x', 'y', and 'z'".to_string(),
        ["x", "y", "z", "t"] => "'x', 'y', 'z', and 't'".to_string(),
        _ => allowed_variables
            .iter()
            .map(|variable| format!("'{variable}'"))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// Appends one validation error to the aggregated error list when present.
fn collect_error(result: &Result<EqRender, String>, errors: &mut Vec<String>) {
    if let Err(error) = result {
        errors.push(error.clone());
    }
}

/// Builds the multiline error message shown by the control window.
fn format_error_summary(errors: &[String]) -> String {
    format!("Error in equation(s):\n{}", errors.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::{format_error_summary, validate_ui_state};
    use crate::app::ui::{EmMode, FieldKind, GridUiState};

    #[test]
    fn validate_ui_state_accepts_polynomial_expression() {
        let mut state = GridUiState::default();
        state.coords_sys.x.eq_str = "x + y * z".to_string();

        assert!(validate_ui_state(&state).is_ok());
    }

    #[test]
    fn validate_ui_state_accepts_trigonometric_expression() {
        let mut state = GridUiState::default();
        state.field.x.eq_str = "sin(x) + cos(y) - z".to_string();

        assert!(validate_ui_state(&state).is_ok());
    }

    #[test]
    fn validate_ui_state_rejects_empty_expression() {
        let mut state = GridUiState::default();
        state.coords_sys.x.eq_str.clear();

        let error = validate_ui_state(&state).unwrap_err();

        assert!(error.contains("Coordinate x: Equation cannot be empty"));
    }

    #[test]
    fn validate_ui_state_uses_scalar_equation_in_scalar_mode() {
        let mut state = GridUiState::default();
        state.field_kind = FieldKind::Scalar;
        state.scalar_field.eq_str = "x * y + z".to_string();
        state.field.x.eq_str = "x + t".to_string();

        assert!(validate_ui_state(&state).is_ok());
    }

    #[test]
    fn validate_ui_state_rejects_unknown_variable() {
        let mut state = GridUiState::default();
        state.field.y.eq_str = "x + t".to_string();

        let error = validate_ui_state(&state).unwrap_err();

        assert!(error.contains("Field Fy: Invalid variable 't'"));
    }

    #[test]
    fn validate_ui_state_accepts_time_only_for_em_equations() {
        let mut state = GridUiState::default();
        state.em.enabled = true;
        state.em.mode = EmMode::Electric;
        state.em.electric_field.x.eq_str = "sin(t) + x".to_string();

        assert!(validate_ui_state(&state).is_ok());
    }

    #[test]
    fn validate_ui_state_ignores_potential_drafts_in_electric_em_mode() {
        let mut state = GridUiState::default();
        state.em.enabled = true;
        state.em.mode = EmMode::Electric;
        state.em.phi.eq_str.clear();
        state.em.vector_potential.x.eq_str = "invalid(".to_string();

        assert!(validate_ui_state(&state).is_ok());
    }

    #[test]
    fn validate_ui_state_ignores_regular_field_drafts_while_em_is_enabled() {
        let mut state = GridUiState::default();
        state.field_kind = FieldKind::Vector;
        state.field.x.eq_str = "invalid(".to_string();
        state.em.enabled = true;
        state.em.mode = EmMode::Electric;
        state.em.electric_field.y.eq_str = "cos(z - t)".to_string();

        assert!(validate_ui_state(&state).is_ok());
    }

    #[test]
    fn format_error_summary_joins_multiple_lines() {
        let message = format_error_summary(&[
            "Coordinate x: bad".to_string(),
            "Field Fz: worse".to_string(),
        ]);

        assert_eq!(
            message,
            "Error in equation(s):\nCoordinate x: bad\nField Fz: worse"
        );
    }
}
