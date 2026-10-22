//! Modification d'une règle existante : définition remplacée, position conservée, idempotente par clé.

use mira_vocabulary::{CURRENT_RULE_SCHEMA_VERSION, IdempotencyKey, MiraError, Rule};

use super::idempotency::{Replay, WriteCommand};
use super::reading::{require_rule, require_view, view_of};
use super::writing::{announce, check_writable, single, with_normalized_scope};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Remplace la règle de même identifiant ; inconnue : `not_found.rule` ; incorrecte : refusée sans écrire
pub(crate) fn handle_update_rule(
    engine: &Engine,
    rule: Rule,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let command = WriteCommand::new(key, "update_rule", &rule)?;
    let rule = with_normalized_scope(Rule {
        schema_version: CURRENT_RULE_SCHEMA_VERSION,
        ..rule
    });
    let (view, written) = engine.database.write(|tx| {
        if let Replay::Replayed { .. } = command.replay(tx)? {
            return Ok((require_view(tx, &rule.id)?, false));
        }
        require_rule(tx, &rule.id)?;
        let world = check_writable(tx, &rule)?;
        tx.update_rule(&rule)?;
        command.record(tx, Some(&rule.id))?;
        Ok((view_of(rule.clone(), &world), true))
    })?;
    if written {
        announce(engine, &view.rule.id)?;
    }
    Ok(single(view))
}
