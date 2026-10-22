//! Validation d'une règle sans l'enregistrer : le statut qu'elle aurait à l'enregistrement.

use mira_vocabulary::{MiraError, Rule};

use crate::api::contract::{Response, RuleResponse};
use crate::api::engine::Engine;
use crate::rule_context::StoredRuleWorld;
use crate::rules::validate::{RuleStatus, validate_rule};

/// Rend `valid`, ou `invalid` avec la raison du refus ; rien n'est écrit ni annoncé
pub(crate) fn handle_validate_rule(engine: &Engine, rule: &Rule) -> Result<Response, MiraError> {
    let world = engine.database.read(StoredRuleWorld::load)?;
    let status = match validate_rule(rule, &world) {
        Ok(()) => RuleStatus::Valid,
        Err(error) => RuleStatus::Invalid {
            reason: error.message,
        },
    };
    Ok(Response::Rule(RuleResponse::Status(status)))
}
