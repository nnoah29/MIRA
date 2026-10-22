//! Portée : comptes inexistants, portée vide, règle sans action de boîte sur tous les comptes.

use crate::world::{valid_rule, world};
use mira_core::rules::validate::validate_rule;
use mira_vocabulary::{AccountId, RuleAction, RuleScope};

/// Un compte inexistant dans la portée est refusé
#[test]
fn unknown_account_is_refused() {
    let mut rule = valid_rule();
    rule.actions = vec![RuleAction::Trash];
    rule.scope = RuleScope::Accounts(vec![AccountId::from_raw("acc-9")]);
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Une portée vide est refusée
#[test]
fn empty_scope_is_refused() {
    let mut rule = valid_rule();
    rule.actions = vec![RuleAction::Trash];
    rule.scope = RuleScope::Accounts(Vec::new());
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Une règle sans action de boîte peut viser tous les comptes
#[test]
fn all_accounts_without_mailbox_action_is_accepted() {
    let mut rule = valid_rule();
    rule.actions = vec![RuleAction::Trash];
    rule.scope = RuleScope::AllAccounts;
    assert!(validate_rule(&rule, &world()).is_ok());
}
