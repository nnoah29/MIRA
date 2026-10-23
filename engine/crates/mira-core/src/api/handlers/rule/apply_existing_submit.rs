//! Effets d'une règle appliquée à l'existant : actions mises en file par compte, libellés locaux posés par fil.

use std::collections::{BTreeMap, BTreeSet};

use mira_vocabulary::{AccountId, ActionKind, EventKind, IdempotencyKey, MessageId, MiraError};

use crate::actions::{BatchItem, SubmitRequest, submit_batch};
use crate::api::engine::Engine;
use crate::events::EventSink;
use crate::labels::refresh_inbox_counters;
use crate::rules::apply::apply_label_changes;
use crate::rules::context::MessageContext;
use crate::rules::outcome::RuleOutcome;

/// Une demande de mise en file par compte pour cette action, sur les messages touchés de ce compte
fn items_for(
    engine: &Engine,
    kind: &ActionKind,
    key: &IdempotencyKey,
    matched: &[MessageContext],
) -> Result<Vec<BatchItem>, MiraError> {
    let mut groups: BTreeMap<AccountId, Vec<MessageId>> = BTreeMap::new();
    for context in matched {
        let summary = &context.summary;
        groups
            .entry(summary.account.clone())
            .or_default()
            .push(summary.id.clone());
    }
    groups
        .into_iter()
        .map(|(account, targets)| {
            Ok(BatchItem {
                quirks: engine.quirks_of(&account)?,
                request: SubmitRequest {
                    account,
                    kind: kind.clone(),
                    targets,
                    key: key.clone(),
                },
            })
        })
        .collect()
}

/// Met en file chaque action de la règle, sous une clé dérivée de celle du client et du rang de l'action
pub(super) fn submit_actions(
    engine: &Engine,
    outcome: &RuleOutcome,
    key: &IdempotencyKey,
    matched: &[MessageContext],
) -> Result<(), MiraError> {
    for (rank, kind) in outcome.actions.iter().enumerate() {
        let ranked = IdempotencyKey(format!("{}:{rank}", key.0));
        submit_batch(
            &engine.queue_context(),
            &items_for(engine, kind, &ranked, matched)?,
        )?;
    }
    engine.wake_replayers();
    Ok(())
}

/// Pose les libellés locaux de la règle sur chaque fil touché, puis annonce le changement s'il y en a un
pub(super) fn apply_labels(
    engine: &Engine,
    outcome: &RuleOutcome,
    matched: &[MessageContext],
) -> Result<(), MiraError> {
    if outcome.label_changes.is_empty() {
        return Ok(());
    }
    let threads: BTreeSet<_> = matched
        .iter()
        .filter_map(|c| c.summary.thread.clone())
        .collect();
    let changed = engine.database.write(|tx| {
        let mut any = false;
        for thread in &threads {
            any |= apply_label_changes(tx, Some(thread), &outcome.label_changes)?;
        }
        Ok(any)
    })?;
    if changed {
        engine.events.emit(EventKind::LocalLabelsChanged)?;
        refresh_inbox_counters(&engine.database, &engine.events, false)?;
    }
    Ok(())
}
