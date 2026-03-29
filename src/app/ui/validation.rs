use crate::app::ui::state::{EqRender, GridUiState, SpacialEqs};
use mathhook_core::Parser;

#[derive(Debug)]
pub(crate) struct ValidatedUiState {
    pub coords_sys: SpacialEqs,
    pub field: SpacialEqs,
}

pub(crate) fn validate_ui_state(state: &GridUiState) -> Result<ValidatedUiState, String> {
    let coord_x = validate_equation("Coordinate x", &state.coords_sys.x.eq_str);
    let coord_y = validate_equation("Coordinate y", &state.coords_sys.y.eq_str);
    let coord_z = validate_equation("Coordinate z", &state.coords_sys.z.eq_str);
    let field_x = validate_equation("Field Fx", &state.field.x.eq_str);
    let field_y = validate_equation("Field Fy", &state.field.y.eq_str);
    let field_z = validate_equation("Field Fz", &state.field.z.eq_str);

    let mut errors = Vec::new();
    collect_error(&coord_x, &mut errors);
    collect_error(&coord_y, &mut errors);
    collect_error(&coord_z, &mut errors);
    collect_error(&field_x, &mut errors);
    collect_error(&field_y, &mut errors);
    collect_error(&field_z, &mut errors);

    if !errors.is_empty() {
        return Err(format_error_summary(&errors));
    }

    Ok(ValidatedUiState {
        coords_sys: SpacialEqs {
            x: coord_x.unwrap(),
            y: coord_y.unwrap(),
            z: coord_z.unwrap(),
        },
        field: SpacialEqs {
            x: field_x.unwrap(),
            y: field_y.unwrap(),
            z: field_z.unwrap(),
        },
    })
}

fn validate_equation(label: &str, eq: &str) -> Result<EqRender, String> {
    if eq.is_empty() {
        return Err(format!("{label}: Equation cannot be empty"));
    }

    let formal_eq = Parser::default()
        .parse(eq)
        .map_err(|error| format!("{label}: Invalid equation: {error}"))?;

    for variable in formal_eq.find_variables() {
        if variable.name() != "x" && variable.name() != "y" && variable.name() != "z" {
            return Err(format!(
                "{label}: Invalid variable '{}'. Only 'x', 'y', and 'z' are allowed.",
                variable.name()
            ));
        }
    }

    Ok(EqRender::new(formal_eq, eq.to_string()))
}

fn collect_error(result: &Result<EqRender, String>, errors: &mut Vec<String>) {
    if let Err(error) = result {
        errors.push(error.clone());
    }
}

fn format_error_summary(errors: &[String]) -> String {
    format!("Error in equation(s):\n{}", errors.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::{format_error_summary, validate_ui_state};
    use crate::app::ui::GridUiState;

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
    fn validate_ui_state_rejects_unknown_variable() {
        let mut state = GridUiState::default();
        state.field.y.eq_str = "x + t".to_string();

        let error = validate_ui_state(&state).unwrap_err();

        assert!(error.contains("Field Fy: Invalid variable 't'"));
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
