//! Boîte d'envoi d'un compte : envois planifiés, en cours, incertains ou échoués (lecture locale).

use mira_vocabulary::{AccountId, ActionKind, ActionState, MiraError, PendingAction};

use crate::accounts::require_account;
use crate::api::contract::{OutboxEntry, Response, WriteResponse};
use crate::api::engine::Engine;
use crate::storage::Tx;

/// Rend un envoi par action `Send` encore ouverte, dans l'ordre de la file
pub(crate) fn handle_list_outbox(
    engine: &Engine,
    account: &AccountId,
) -> Result<Response, MiraError> {
    let open = [
        ActionState::Pending,
        ActionState::InProgress,
        ActionState::Uncertain,
        ActionState::Failed,
    ];
    let entries = engine.database.read(|tx| {
        require_account(tx, account)?;
        let mut entries = Vec::new();
        for action in tx.list_actions(account, &open)? {
            entries.extend(entry_of(tx, &action)?);
        }
        Ok(entries)
    })?;
    Ok(Response::Write(WriteResponse::Outbox(entries)))
}

/// Fiche d'envoi d'une action, ou rien si ce n'est pas un envoi
fn entry_of(tx: &Tx<'_>, action: &PendingAction) -> Result<Option<OutboxEntry>, MiraError> {
    let ActionKind::Send { draft } = &action.kind else {
        return Ok(None);
    };
    let subject = tx
        .get_draft(draft)?
        .map(|found| found.subject)
        .unwrap_or_default();
    Ok(Some(OutboxEntry {
        action: action.id.clone(),
        draft: draft.clone(),
        state: action.state,
        scheduled_for: action.not_before,
        subject,
        last_error: action.last_error.clone(),
    }))
}
