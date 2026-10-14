//! Regroupement des messages ciblés par compte : une demande de mise en file par compte.

use mira_vocabulary::{AccountId, ActionKind, IdempotencyKey, MessageId, MiraError};

use crate::actions::{BatchItem, SubmitRequest};
use crate::api::engine::Engine;
use crate::storage::repositories::messages::message_not_found;

/// Une demande par compte concerné, dans l'ordre d'apparition des messages
pub(super) fn batch_items(
    engine: &Engine,
    kind: &ActionKind,
    key: &IdempotencyKey,
    messages: Vec<MessageId>,
) -> Result<Vec<BatchItem>, MiraError> {
    let mut groups: Vec<(AccountId, Vec<MessageId>)> = Vec::new();
    for message in messages {
        let account = account_of_message(engine, &message)?;
        match groups.iter_mut().find(|(known, _)| *known == account) {
            Some((_, targets)) => targets.push(message),
            None => groups.push((account, vec![message])),
        }
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

/// Compte auquel appartient le message, ou `not_found.message`
fn account_of_message(engine: &Engine, message: &MessageId) -> Result<AccountId, MiraError> {
    engine.database.read(|tx| {
        tx.get_message(message)?
            .map(|summary| summary.account)
            .ok_or_else(|| message_not_found(message))
    })
}
