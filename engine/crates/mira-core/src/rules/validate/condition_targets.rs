//! Cibles des conditions « dans la boîte » et « reçu sur le compte » : existantes et dans la portée.

use mira_vocabulary::{AccountId, ConditionTest, MiraError, Rule, RuleScope};

use super::world::RuleWorld;
use crate::actions::errors::input::invalid_parameter;

/// Vrai si la portée de la règle couvre le compte.
fn scope_covers(scope: &RuleScope, account: &AccountId) -> bool {
    match scope {
        RuleScope::AllAccounts => true,
        RuleScope::Accounts(accounts) => accounts.contains(account),
    }
}

/// Vrai si la cible du test existe et relève de la portée ; les autres tests n'ont pas de cible.
fn target_is_reachable(test: &ConditionTest, scope: &RuleScope, world: &dyn RuleWorld) -> bool {
    match test {
        ConditionTest::InMailbox { mailbox } => world
            .mailbox_account(mailbox)
            .is_some_and(|owner| scope_covers(scope, &owner)),
        ConditionTest::FromAccount { account } => {
            world.account_exists(account) && scope_covers(scope, account)
        }
        _ => true,
    }
}

/// Refuse une condition dont la boîte ou le compte visé n'existe pas ou sort de la portée.
pub(super) fn check_condition_targets(rule: &Rule, world: &dyn RuleWorld) -> Result<(), MiraError> {
    let unreachable = rule
        .alternatives
        .iter()
        .flat_map(|group| group.conditions.iter())
        .any(|condition| !target_is_reachable(&condition.test, &rule.scope, world));
    if unreachable {
        return Err(invalid_parameter(
            "une condition vise une boîte ou un compte qui n'existe plus ou sort de la portée",
        ));
    }
    Ok(())
}
