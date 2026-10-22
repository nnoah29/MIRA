//! Réordonnancement des règles, idempotent par clé ; seules les règles déplacées sont annoncées.

use mira_vocabulary::{IdempotencyKey, MiraError, RuleId};

use super::idempotency::{Replay, WriteCommand};
use super::reading::all_views;
use super::writing::announce;
use crate::api::contract::{Response, RuleResponse};
use crate::api::engine::Engine;

/// Applique l'ordre donné, qui doit citer chaque règle exactement une fois, et rend la liste réordonnée
pub(crate) fn handle_reorder_rules(
    engine: &Engine,
    order: &[RuleId],
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let command = WriteCommand::new(key, "reorder_rules", &order)?;
    let (views, moved) = engine.database.write(|tx| {
        if let Replay::Replayed { .. } = command.replay(tx)? {
            return Ok((all_views(tx)?, Vec::new()));
        }
        let before: Vec<RuleId> = tx.list_rules()?.into_iter().map(|rule| rule.id).collect();
        tx.reorder_rules(order)?;
        command.record(tx, None)?;
        Ok((all_views(tx)?, moved_rules(&before, order)))
    })?;
    for id in &moved {
        announce(engine, id)?;
    }
    Ok(Response::Rule(RuleResponse::Rules(views)))
}

/// Règles dont la position a changé, dans le nouvel ordre
fn moved_rules(before: &[RuleId], after: &[RuleId]) -> Vec<RuleId> {
    after
        .iter()
        .zip(before.iter())
        .filter(|(new, old)| new != old)
        .map(|(new, _)| new.clone())
        .collect()
}
