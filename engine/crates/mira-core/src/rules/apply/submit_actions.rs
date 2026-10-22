//! Évaluation d'un message en attente et soumission de ses actions avec des clés d'idempotence déterministes.

use mira_vocabulary::{AccountId, EventKind, IdempotencyKey, MessageId, MiraError, ProviderQuirks};

use super::context_loader::load_for_message;
use super::label_changes::apply_label_changes;
use crate::actions::{QueueContext, SubmitRequest, submit};
use crate::labels::refresh_inbox_counters;
use crate::rules::evaluate::evaluate;
use crate::rules::outcome::RuleOutcome;

/// Clé dérivée du message et du rang de l'action : un rejeu rend les mêmes actions (invariant 18)
fn action_key(message: &MessageId, rank: usize) -> IdempotencyKey {
    IdempotencyKey(format!("rule:{}:{rank}", message.as_str()))
}

/// Soumet chaque action du résultat, puis annonce chaque règle déclenchée
fn submit_outcome(
    queue: &QueueContext<'_>,
    quirks: &ProviderQuirks,
    account: &AccountId,
    message: &MessageId,
    outcome: &RuleOutcome,
) -> Result<(), MiraError> {
    for (rank, kind) in outcome.actions.iter().enumerate() {
        let request = SubmitRequest {
            account: account.clone(),
            kind: kind.clone(),
            targets: vec![message.clone()],
            key: action_key(message, rank),
        };
        submit(queue, quirks, request)?;
    }
    for rule in &outcome.matched {
        queue.events.emit(EventKind::RuleApplied {
            account: account.clone(),
            rule: rule.clone(),
            message: message.clone(),
        })?;
    }
    Ok(())
}

/// Évalue les règles sur le message, soumet les actions, pose les libellés locaux, émet `RuleApplied` puis retire le message de la file d'attente
pub fn apply_rules_with(
    queue: &QueueContext<'_>,
    quirks: &ProviderQuirks,
    message: &MessageId,
) -> Result<RuleOutcome, MiraError> {
    let Some(loaded) = load_for_message(queue.db, message, queue.clock.now())? else {
        queue.db.write(|tx| tx.remove_pending(message))?;
        return Ok(RuleOutcome::default());
    };
    let outcome = evaluate(&loaded.rules, &loaded.context);
    let account = &loaded.context.summary.account;
    submit_outcome(queue, quirks, account, message, &outcome)?;
    let thread = loaded.context.summary.thread.as_ref();
    let labeled = queue.db.write(|tx| {
        let changed = apply_label_changes(tx, thread, &outcome.label_changes)?;
        tx.remove_pending(message)?;
        Ok(changed)
    })?;
    if labeled {
        queue.events.emit(EventKind::LocalLabelsChanged)?;
        refresh_inbox_counters(queue.db, queue.events, false)?;
    }
    Ok(outcome)
}
