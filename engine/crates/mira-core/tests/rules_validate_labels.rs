#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Validation des règles qui visent un libellé local : il doit exister, la portée n'importe pas.

mod support_rules;
#[path = "rules_validate/world.rs"]
mod world;

use mira_core::rules::validate::{RuleStatus, rule_status, validate_rule};
use mira_vocabulary::{ConditionTest, LocalLabelId, Rule, RuleAction, RuleScope};
use support_rules::builders::{cond, rule};
use world::{FakeWorld, world};

/// Règle « tous les comptes » qui pose le libellé voulu
fn adds(raw: &str) -> Rule {
    let label = LocalLabelId::from_raw(raw);
    let test = ConditionTest::HasLocalLabel {
        label: label.clone(),
    };
    rule(
        "r",
        vec![vec![cond(test)]],
        vec![RuleAction::AddLocalLabel { label }],
    )
}

/// Un libellé connu est accepté, avec « tous les comptes » comme avec un compte
#[test]
fn known_label_is_accepted_for_any_scope() {
    let rule = adds("lab-1");
    assert!(validate_rule(&rule, &world()).is_ok());
    let mut scoped = adds("lab-1");
    scoped.scope = RuleScope::Accounts(vec![mira_vocabulary::AccountId::from_raw("acc-1")]);
    assert!(validate_rule(&scoped, &world()).is_ok());
}

/// Un libellé inconnu est refusé, qu'il soit dans une condition ou dans une action
#[test]
fn unknown_label_is_refused_in_conditions_and_actions() {
    assert!(validate_rule(&adds("zz"), &world()).is_err());
    let removes = RuleAction::RemoveLocalLabel {
        label: LocalLabelId::from_raw("zz"),
    };
    let mut in_action = adds("lab-1");
    in_action.actions = vec![removes];
    assert!(validate_rule(&in_action, &world()).is_err());
}

/// Un libellé supprimé ensuite rend la règle invalide à la lecture
#[test]
fn status_is_invalid_when_label_disappears() {
    let rule = adds("lab-1");
    assert_eq!(rule_status(&rule, &world()), RuleStatus::Valid);
    let shrunk = FakeWorld {
        labels: vec![],
        ..world()
    };
    assert!(matches!(
        rule_status(&rule, &shrunk),
        RuleStatus::Invalid { .. }
    ));
}
