//! Activation ou désactivation d'une règle, idempotente par clé.

use mira_vocabulary::{IdempotencyKey, MiraError, RuleId};

use super::idempotency::{Replay, WriteCommand};
use super::reading::{require_rule, require_view, view_of};
use super::writing::{announce, check_writable, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::rule_context::StoredRuleWorld;

/// Active (après contrôle complet de la règle) ou désactive (toujours permis) une règle existante
pub(crate) fn handle_set_rule_enabled(
    engine: &Engine,
    id: &RuleId,
    enabled: bool,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let command = WriteCommand::new(key, "set_rule_enabled", &(id, enabled))?;
    let (view, written) = engine.database.write(|tx| {
        if let Replay::Replayed { .. } = command.replay(tx)? {
            return Ok((require_view(tx, id)?, false));
        }
        let mut rule = require_rule(tx, id)?;
        rule.enabled = enabled;
        let world = if enabled {
            check_writable(tx, &rule)?
        } else {
            StoredRuleWorld::load(tx)?
        };
        tx.set_enabled(id, enabled)?;
        command.record(tx, Some(id))?;
        Ok((view_of(rule, &world), true))
    })?;
    if written {
        announce(engine, id)?;
    }
    Ok(single(view))
}
