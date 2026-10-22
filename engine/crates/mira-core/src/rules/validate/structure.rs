//! Structure minimale d'une règle : alternatives, conditions et actions présentes.

use mira_vocabulary::{MiraError, Rule};

use crate::actions::errors::input::invalid_parameter;

/// Exige au moins une alternative non vide et au moins une action.
pub(super) fn check_structure(rule: &Rule) -> Result<(), MiraError> {
    if rule.alternatives.is_empty() {
        return Err(invalid_parameter("la règle n'a aucune alternative"));
    }
    if rule.alternatives.iter().any(|g| g.conditions.is_empty()) {
        return Err(invalid_parameter("une alternative n'a aucune condition"));
    }
    if rule.actions.is_empty() {
        return Err(invalid_parameter("la règle n'a aucune action"));
    }
    Ok(())
}
