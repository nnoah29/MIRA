//! Plafonds : alternatives, conditions par groupe, actions et taille des valeurs.

use crate::support_rules::probe::{cond, text};
use crate::world::{valid_rule, world};
use mira_core::rules::validate::{
    MAX_ACTIONS, MAX_ALTERNATIVES, MAX_CONDITIONS_PER_GROUP, MAX_VALUE_LEN, validate_rule,
};
use mira_vocabulary::{ConditionTest, RuleAction, TextOp};

/// Trop d'alternatives
#[test]
fn too_many_alternatives_are_refused() {
    let mut rule = valid_rule();
    let group = rule.alternatives[0].clone();
    rule.alternatives = vec![group; MAX_ALTERNATIVES + 1];
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Trop de conditions dans un groupe
#[test]
fn too_many_conditions_are_refused() {
    let mut rule = valid_rule();
    let condition = rule.alternatives[0].conditions[0].clone();
    rule.alternatives[0].conditions = vec![condition; MAX_CONDITIONS_PER_GROUP + 1];
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Trop d'actions
#[test]
fn too_many_actions_are_refused() {
    let mut rule = valid_rule();
    rule.actions = vec![RuleAction::SuppressNotification; MAX_ACTIONS + 1];
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Valeur de filtre trop longue
#[test]
fn too_long_value_is_refused() {
    let mut rule = valid_rule();
    rule.alternatives[0].conditions[0] = cond(ConditionTest::Subject {
        matcher: text(TextOp::Contains, &"a".repeat(MAX_VALUE_LEN + 1)),
    });
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Les plafonds exacts sont acceptés
#[test]
fn limits_exactly_reached_are_accepted() {
    let mut rule = valid_rule();
    let group = rule.alternatives[0].clone();
    rule.alternatives = vec![group; MAX_ALTERNATIVES];
    rule.actions = vec![RuleAction::SuppressNotification; MAX_ACTIONS];
    assert!(validate_rule(&rule, &world()).is_ok());
}
