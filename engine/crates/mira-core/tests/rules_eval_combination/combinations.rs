//! ET dans un groupe, OU entre groupes : les cinq formes du brief.

use crate::support_rules::fixtures::base_context;
use crate::support_rules::probe::{no, rule, yes};
use mira_core::rules::evaluate::evaluate;
use mira_vocabulary::{Condition, RuleAction};

/// Vrai si une règle à ces groupes se déclenche
fn fires(groups: Vec<Vec<Condition>>) -> bool {
    let rules = [rule("r", groups, vec![RuleAction::Archive])];
    !evaluate(&rules, &base_context()).matched.is_empty()
}

/// Une seule condition décide
#[test]
fn single_condition_decides() {
    assert!(fires(vec![vec![yes()]]));
    assert!(!fires(vec![vec![no()]]));
}

/// ET dans un groupe : toutes les conditions doivent être vraies
#[test]
fn and_inside_group_requires_all() {
    assert!(fires(vec![vec![yes(), yes()]]));
    assert!(!fires(vec![vec![yes(), no()]]));
    assert!(fires(vec![vec![yes(), yes(), yes()]]));
    assert!(!fires(vec![vec![yes(), yes(), no()]]));
}

/// OU entre groupes : un groupe vrai suffit
#[test]
fn or_between_groups_requires_one() {
    assert!(fires(vec![vec![no()], vec![yes()]]));
    assert!(!fires(vec![vec![no()], vec![no()]]));
}

/// `A || C && B` : le ET lie plus fort que le OU
#[test]
fn and_binds_tighter_than_or() {
    assert!(fires(vec![vec![no()], vec![yes(), yes()]]));
    assert!(!fires(vec![vec![no()], vec![yes(), no()]]));
    assert!(fires(vec![vec![yes()], vec![no(), no()]]));
}
