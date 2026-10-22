//! Accroche de la synchronisation aux règles : évaluation d'un message qui vient d'arriver.

use mira_vocabulary::{MessageId, MiraError};

use crate::actions::QueueContext;
use crate::rules::apply::apply_rules_with;
use crate::rules::outcome::RuleOutcome;
use crate::sync::context::SyncContext;

/// Évalue les règles sur un message arrivé en synchronisation et soumet leurs actions
pub fn apply_rules_to_message(
    ctx: &SyncContext,
    message: &MessageId,
) -> Result<RuleOutcome, MiraError> {
    let queue = QueueContext {
        db: &ctx.db,
        clock: ctx.clock.as_ref(),
        ids: ctx.ids.as_ref(),
        events: ctx.events.as_ref(),
    };
    apply_rules_with(&queue, &ctx.provider.descriptor().quirks, message)
}

/// Vrai si les règles demandent le silence ; une erreur est journalisée sans interrompre la synchronisation
pub fn silenced_by_rules(ctx: &SyncContext, message: &MessageId) -> bool {
    match apply_rules_to_message(ctx, message) {
        Ok(outcome) => outcome.silenced,
        Err(error) => {
            tracing::warn!(code = %error.code, "règles non appliquées à un message arrivé");
            false
        }
    }
}
