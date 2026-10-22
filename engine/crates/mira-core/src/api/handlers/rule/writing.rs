//! Garde-fous communs aux écritures de règles : contrôle avant écriture, annonce, réponse.

use mira_vocabulary::{EventKind, MiraError, Rule, RuleId, RuleScope};

use crate::accounts::require_account;
use crate::api::contract::{Response, RuleResponse, RuleView};
use crate::api::engine::Engine;
use crate::events::EventSink;
use crate::rule_context::StoredRuleWorld;
use crate::rules::validate::validate_rule;
use crate::storage::Tx;

/// Refuse avant toute écriture : compte de la portée inconnu (`not_found.account`), puis règle incorrecte
pub(super) fn check_writable(tx: &Tx<'_>, rule: &Rule) -> Result<StoredRuleWorld, MiraError> {
    if let RuleScope::Accounts(accounts) = &rule.scope {
        for account in accounts {
            require_account(tx, account)?;
        }
    }
    let world = StoredRuleWorld::load(tx)?;
    validate_rule(rule, &world)?;
    Ok(world)
}

/// Règle dont la portée est triée et sans doublon, telle que le stockage la rendra à la lecture
pub(super) fn with_normalized_scope(rule: Rule) -> Rule {
    let scope = match rule.scope {
        RuleScope::Accounts(mut accounts) => {
            accounts.sort();
            accounts.dedup();
            RuleScope::Accounts(accounts)
        }
        RuleScope::AllAccounts => RuleScope::AllAccounts,
    };
    Rule { scope, ..rule }
}

/// Annonce qu'une règle a changé (création, modification, activation, position, suppression)
pub(super) fn announce(engine: &Engine, rule: &RuleId) -> Result<(), MiraError> {
    engine
        .events
        .emit(EventKind::RuleChanged { rule: rule.clone() })?;
    Ok(())
}

/// Réponse portant une seule règle et son statut
pub(super) fn single(view: RuleView) -> Response {
    Response::Rule(RuleResponse::Rule(view))
}
