//! Contrôles qui dépendent du monde : comptes de la portée et boîtes visées par les actions.

use mira_vocabulary::{AccountId, MailboxId, MiraError, Rule, RuleAction, RuleScope};

use super::world::RuleWorld;
use crate::actions::errors::input::invalid_parameter;

/// Boîte nommée par l'action, s'il y en a une.
fn named_mailbox(action: &RuleAction) -> Option<&MailboxId> {
    match action {
        RuleAction::MoveTo { mailbox }
        | RuleAction::AddToMailbox { mailbox }
        | RuleAction::RemoveFromMailbox { mailbox } => Some(mailbox),
        _ => None,
    }
}

/// Vrai si l'action dépend d'une boîte du compte (boîte nommée ou boîte d'archive).
fn depends_on_account_mailbox(action: &RuleAction) -> bool {
    named_mailbox(action).is_some() || matches!(action, RuleAction::Archive)
}

/// Vérifie la portée puis les boîtes visées ; sert à l'enregistrement et à la lecture.
pub(super) fn check_world(rule: &Rule, world: &dyn RuleWorld) -> Result<(), MiraError> {
    check_scope_accounts(rule, world)?;
    if !rule.actions.iter().any(depends_on_account_mailbox) {
        return Ok(());
    }
    let account = single_account(&rule.scope)?;
    for mailbox in rule.actions.iter().filter_map(named_mailbox) {
        if world.mailbox_account(mailbox).as_ref() != Some(account) {
            return Err(invalid_parameter(
                "une boîte visée n'existe plus ou n'appartient pas au compte de la règle",
            ));
        }
    }
    Ok(())
}

/// Refuse une portée vide ou qui nomme un compte inexistant.
fn check_scope_accounts(rule: &Rule, world: &dyn RuleWorld) -> Result<(), MiraError> {
    let RuleScope::Accounts(accounts) = &rule.scope else {
        return Ok(());
    };
    if accounts.is_empty() {
        return Err(invalid_parameter("la portée de la règle est vide"));
    }
    if accounts
        .iter()
        .any(|account| !world.account_exists(account))
    {
        return Err(invalid_parameter("un compte de la portée n'existe pas"));
    }
    Ok(())
}

/// Compte unique de la portée ; les actions de boîte l'exigent.
fn single_account(scope: &RuleScope) -> Result<&AccountId, MiraError> {
    match scope {
        RuleScope::Accounts(accounts) if accounts.len() == 1 => Ok(&accounts[0]),
        _ => Err(invalid_parameter(
            "une action de boîte exige une règle limitée à un seul compte",
        )),
    }
}
