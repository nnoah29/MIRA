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

/// Nombre maximal de valeurs d'un filtre (liste `in_list` comprise).
pub const MAX_VALUES_PER_MATCHER: usize = 50;

/// Longueur maximale, en octets, du nom d'une règle.
pub const MAX_NAME_LEN: usize = 200;

/// Refuse une règle qui dépasse un plafond de taille.
pub(super) fn check_limits(rule: &Rule) -> Result<(), MiraError> {
    if rule.name.len() > MAX_NAME_LEN {
        return Err(invalid_parameter("le nom de la règle est trop long"));
    }
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
    check_values(rule)
}

/// Refuse un filtre qui a trop de valeurs ou une valeur plus longue que le plafond.
fn check_values(rule: &Rule) -> Result<(), MiraError> {
    let matchers = rule_matchers(rule);
    if matchers
        .iter()
        .any(|matcher| matcher.values.len() > MAX_VALUES_PER_MATCHER)
    {
        return Err(invalid_parameter("un filtre a trop de valeurs"));
    }
    let too_long = matchers
        .iter()
        .flat_map(|matcher| matcher.values.iter())
        .any(|value| value.len() > MAX_VALUE_LEN);
    if too_long {
        return Err(invalid_parameter("une valeur de filtre est trop longue"));
    }
    Ok(())
}
