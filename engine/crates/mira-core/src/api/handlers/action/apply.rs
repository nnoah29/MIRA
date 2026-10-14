//! Application d'une action à un lot de messages : une action par compte, mises en file ensemble.

use mira_vocabulary::{ActionKind, IdempotencyKey, MiraError};

use super::grouping::batch_items;
use super::selection::resolve_selection;
use crate::actions::errors::input::invalid_parameter;
use crate::actions::submit_batch;
use crate::api::contract::{Response, TargetSelection, WriteResponse};
use crate::api::engine::Engine;

/// Met l'action en file pour tous les messages ciblés, tout ou rien, et rend les identifiants sans attendre le réseau
pub(crate) fn handle_apply_action(
    engine: &Engine,
    kind: &ActionKind,
    targets: TargetSelection,
    key: &IdempotencyKey,
) -> Result<Response, MiraError> {
    require_message_action(kind)?;
    let messages = resolve_selection(engine, targets)?;
    let items = batch_items(engine, kind, key, messages)?;
    let ids = submit_batch(&engine.queue_context(), &items)?;
    engine.wake_replayers();
    Ok(Response::Write(WriteResponse::Actions(ids)))
}

/// Refuse les actions qui ne portent pas sur des messages : boîtes et envoi ont leurs propres requêtes
fn require_message_action(kind: &ActionKind) -> Result<(), MiraError> {
    match kind {
        ActionKind::MarkRead { .. }
        | ActionKind::MarkFlagged { .. }
        | ActionKind::Move { .. }
        | ActionKind::AddToMailbox { .. }
        | ActionKind::RemoveFromMailbox { .. }
        | ActionKind::Archive
        | ActionKind::Trash
        | ActionKind::DeletePermanently
        | ActionKind::MarkSpam { .. } => Ok(()),
        ActionKind::CreateMailbox { .. }
        | ActionKind::RenameMailbox { .. }
        | ActionKind::DeleteMailbox { .. }
        | ActionKind::EmptyMailbox { .. }
        | ActionKind::SaveDraft { .. }
        | ActionKind::Send { .. } => Err(invalid_parameter(
            "cette action ne s'applique pas à des messages",
        )),
    }
}
