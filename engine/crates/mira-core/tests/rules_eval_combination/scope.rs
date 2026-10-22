//! Portée par compte et règles désactivées.

use crate::support_rules::fixtures::base_context;
use crate::support_rules::probe::{cond, rule, scoped};
use mira_core::rules::evaluate::evaluate;
use mira_vocabulary::{ConditionTest, RuleAction, RuleId};

/// Une règle `Accounts(liste)` ne vise que les comptes listés
#[test]
fn scope_filters_by_account() {
    let truth = cond(ConditionTest::HasAttachment);
    let rules = [
        scoped("other", &["acc-2"], truth.clone()),
        scoped("mine", &["acc-2", "acc-1"], truth),
    ];
    let outcome = evaluate(&rules, &base_context());
    assert_eq!(outcome.matched, vec![RuleId::from_raw("mine")]);
}

/// Une portée `Accounts` vide ne vise aucun compte
#[test]
fn empty_account_list_matches_nothing() {
    let rules = [scoped("none", &[], cond(ConditionTest::HasAttachment))];
    assert!(evaluate(&rules, &base_context()).matched.is_empty());
}

/// `AllAccounts` vise tous les comptes
#[test]
fn all_accounts_scope_matches_any_account() {
    let rules = [rule(
        "all",
        vec![vec![cond(ConditionTest::HasAttachment)]],
        vec![RuleAction::Archive],
    )];
    assert_eq!(evaluate(&rules, &base_context()).matched.len(), 1);
}

/// Une règle désactivée est ignorée
#[test]
fn disabled_rule_is_skipped() {
    let mut disabled = rule(
        "off",
        vec![vec![cond(ConditionTest::HasAttachment)]],
        vec![RuleAction::Archive],
    );
    disabled.enabled = false;
    let outcome = evaluate(&[disabled], &base_context());
    assert!(outcome.matched.is_empty());
    assert!(outcome.actions.is_empty());
}
