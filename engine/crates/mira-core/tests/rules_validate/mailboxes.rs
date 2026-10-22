//! Actions de boîte : un seul compte, boîte appartenant à ce compte.

use crate::world::{valid_rule, with_action, world};
use mira_core::rules::validate::validate_rule;
use mira_vocabulary::{AccountId, MailboxId, RuleAction, RuleScope};

/// Une action de boîte sur tous les comptes est refusée
#[test]
fn mailbox_action_on_all_accounts_is_refused() {
    let mut rule = valid_rule();
    rule.scope = RuleScope::AllAccounts;
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Une action de boîte sur plusieurs comptes est refusée
#[test]
fn mailbox_action_on_several_accounts_is_refused() {
    let mut rule = valid_rule();
    rule.scope = RuleScope::Accounts(vec![
        AccountId::from_raw("acc-1"),
        AccountId::from_raw("acc-2"),
    ]);
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Archive dépend de la boîte d'archive du compte : refusée sur tous les comptes
#[test]
fn archive_on_all_accounts_is_refused() {
    let mut rule = with_action(RuleAction::Archive);
    rule.scope = RuleScope::AllAccounts;
    assert!(validate_rule(&rule, &world()).is_err());
}

/// Une boîte d'un autre compte est refusée, pour chaque action de boîte
#[test]
fn mailbox_of_another_account_is_refused() {
    let other = MailboxId::from_raw("mb-2");
    for action in [
        RuleAction::MoveTo {
            mailbox: other.clone(),
        },
        RuleAction::AddToMailbox {
            mailbox: other.clone(),
        },
        RuleAction::RemoveFromMailbox { mailbox: other },
    ] {
        assert!(validate_rule(&with_action(action), &world()).is_err());
    }
}

/// Une boîte inconnue est refusée
#[test]
fn unknown_mailbox_is_refused() {
    let action = RuleAction::AddToMailbox {
        mailbox: MailboxId::from_raw("mb-9"),
    };
    assert!(validate_rule(&with_action(action), &world()).is_err());
}

/// Archive sur un compte unique est acceptée
#[test]
fn archive_on_single_account_is_accepted() {
    assert!(validate_rule(&with_action(RuleAction::Archive), &world()).is_ok());
}
