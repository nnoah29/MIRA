//! Plafonds de taille d'une règle : alternatives, conditions, actions et valeurs.

use mira_vocabulary::{MiraError, Rule};

use super::matchers::rule_matchers;
use crate::actions::errors::input::invalid_parameter;

/// Nombre maximal d'alternatives (OU) par règle.
pub const MAX_ALTERNATIVES: usize = 20;

/// Nombre maximal de conditions (ET) par alternative.
pub const MAX_CONDITIONS_PER_GROUP: usize = 20;

/// Nombre maximal d'actions par règle.
pub const MAX_ACTIONS: usize = 20;

/// Longueur maximale, en octets, d'une valeur de filtre.
pub const MAX_VALUE_LEN: usize = 512;

/// Longueur maximale, en octets, d'une expression régulière.
pub const MAX_REGEX_LEN: usize = 256;

/// Refuse une règle qui dépasse un plafond de taille.
pub(super) fn check_limits(rule: &Rule) -> Result<(), MiraError> {
    if rule.alternatives.len() > MAX_ALTERNATIVES {
        return Err(invalid_parameter("trop d'alternatives dans la règle"));
    }
    if rule
        .alternatives
        .iter()
        .any(|g| g.conditions.len() > MAX_CONDITIONS_PER_GROUP)
    {
        return Err(invalid_parameter("trop de conditions dans une alternative"));
    }
    if rule.actions.len() > MAX_ACTIONS {
        return Err(invalid_parameter("trop d'actions dans la règle"));
    }
    check_value_lengths(rule)
}

/// Refuse une valeur de filtre plus longue que le plafond.
fn check_value_lengths(rule: &Rule) -> Result<(), MiraError> {
    let too_long = rule_matchers(rule)
        .iter()
        .flat_map(|matcher| matcher.values.iter())
        .any(|value| value.len() > MAX_VALUE_LEN);
    if too_long {
        return Err(invalid_parameter("une valeur de filtre est trop longue"));
    }
    Ok(())
}
