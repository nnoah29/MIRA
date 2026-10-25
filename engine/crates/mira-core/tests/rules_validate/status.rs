//! Statut calculé à la lecture : valide, ou invalide avec une raison.

use crate::world::{FakeWorld, valid_rule, world};
use mira_core::rules::validate::{RuleStatus, rule_status};
use mira_vocabulary::{AccountId, RuleAction, RuleScope};

/// Une règle correcte est valide
#[test]
fn status_is_valid_for_correct_rule() {
    assert_eq!(rule_status(&valid_rule(), &world()), RuleStatus::Valid);
}

/// Une boîte disparue rend la règle invalide
#[test]
fn status_is_invalid_when_mailbox_disappears() {
    let shrunk = FakeWorld {
        accounts: vec!["acc-1", "acc-2"],
        mailboxes: vec![("mb-2", "acc-2")],
        labels: vec![],
        templates: vec![],
    };
    assert!(matches!(
        rule_status(&valid_rule(), &shrunk),
        RuleStatus::Invalid { .. }
    ));
}

/// Une portée vide rend la règle invalide, jamais « tous les comptes »
#[test]
fn status_is_invalid_when_scope_is_empty() {
    let mut rule = valid_rule();
    rule.actions = vec![RuleAction::Trash];
    rule.scope = RuleScope::Accounts(Vec::new());
    assert!(matches!(
        rule_status(&rule, &world()),
        RuleStatus::Invalid { .. }
    ));
}

/// Un compte supprimé rend la règle invalide
#[test]
fn status_is_invalid_when_account_is_gone() {
    let mut rule = valid_rule();
    rule.scope = RuleScope::Accounts(vec![AccountId::from_raw("acc-9")]);
    assert!(matches!(
        rule_status(&rule, &world()),
        RuleStatus::Invalid { .. }
    ));
}

/// Le statut se sérialise avec une étiquette `type` en snake_case
#[test]
fn status_serializes_with_snake_case_tag() {
    let invalid = RuleStatus::Invalid {
        reason: "x".to_string(),
    };
    let json = serde_json::to_string(&invalid).unwrap();
    assert!(json.contains("\"type\":\"invalid\""));
    let valid = serde_json::to_string(&RuleStatus::Valid).unwrap();
    assert!(valid.contains("\"valid\""));
}
