//! Suppression d'une règle, idempotente par clé : rejouée, elle répond encore « fait ».

use mira_vocabulary::{IdempotencyKey, MiraError, RuleId};

use super::idempotency::{Replay, WriteCommand};
use super::writing::announce;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Supprime la règle et ses rattachements de comptes ; inconnue : `not_found.rule`
pub(crate) fn handle_delete_rule(
    engine: &Engine,
    id: &RuleId,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let command = WriteCommand::new(key, "delete_rule", id)?;
    let written = engine.database.write(|tx| {
        if let Replay::Replayed { .. } = command.replay(tx)? {
            return Ok(false);
        }
        tx.delete_rule(id)?;
        command.record(tx, Some(id))?;
        Ok(true)
    })?;
    if written {
        announce(engine, id)?;
    }
    Ok(Response::Done)
}
