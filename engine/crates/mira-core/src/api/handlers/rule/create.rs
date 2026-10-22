//! Création d'une règle : contrôlée avant écriture, placée en dernière position, idempotente par clé.

use mira_vocabulary::{CURRENT_RULE_SCHEMA_VERSION, IdempotencyKey, MiraError, Rule, RuleId};

use super::idempotency::{Replay, WriteCommand};
use super::reading::{require_view, view_of};
use super::writing::{announce, check_writable, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Crée la règle sous un identifiant du moteur ; la même clé rejouée rend la même règle sans rien écrire
pub(crate) fn handle_create_rule(
    engine: &Engine,
    rule: Rule,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let command = WriteCommand::new(key, "create_rule", &rule)?;
    let fresh = Rule {
        id: RuleId::from_raw(engine.config.ids.next_id()),
        schema_version: CURRENT_RULE_SCHEMA_VERSION,
        ..rule
    };
    let (view, written) = engine.database.write(|tx| {
        if let Replay::Replayed { rule: Some(id) } = command.replay(tx)? {
            return Ok((require_view(tx, &id)?, false));
        }
        let world = check_writable(tx, &fresh)?;
        tx.insert_rule(&fresh)?;
        command.record(tx, Some(&fresh.id))?;
        Ok((view_of(fresh.clone(), &world), true))
    })?;
    if written {
        announce(engine, &view.rule.id)?;
    }
    Ok(single(view))
}
