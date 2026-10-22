//! Structure minimale : alternatives, conditions et actions présentes.

use crate::world::{valid_rule, world};
use mira_core::rules::validate::validate_rule;

/// Une règle correcte est acceptée
#[test]
fn valid_rule_is_accepted() {
    assert!(validate_rule(&valid_rule(), &world()).is_ok());
}

/// Une alternative sans condition est refusée
#[test]
fn empty_alternative_is_refused() {
    let mut rule = valid_rule();
    rule.alternatives[0].conditions.clear();
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Une règle sans alternative est refusée
#[test]
fn no_alternative_is_refused() {
    let mut rule = valid_rule();
    rule.alternatives.clear();
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Une règle sans action est refusée
#[test]
fn no_action_is_refused() {
    let mut rule = valid_rule();
    rule.actions.clear();
    assert!(validate_rule(&rule, &world()).is_err());
}
