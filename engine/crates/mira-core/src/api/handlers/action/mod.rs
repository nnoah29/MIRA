//! Domaine « actions » : actions sur les messages, annulation, écriture sur les boîtes.

mod apply;
mod create_mailbox;
mod delete_mailbox;
mod empty_mailbox;
mod grouping;
mod mailbox_submit;
mod rename_mailbox;
mod selection;
mod undo;

use mira_vocabulary::MiraError;

use crate::api::contract::{ActionRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine actions
pub(crate) async fn route(engine: &Engine, request: ActionRequest) -> Result<Response, MiraError> {
    match request {
        ActionRequest::ApplyAction {
            kind,
            targets,
            idempotency_key,
        } => apply::handle_apply_action(engine, &kind, targets, &idempotency_key),
        ActionRequest::UndoAction { action } => undo::handle_undo_action(engine, &action),
        ActionRequest::CreateMailbox {
            account,
            name,
            parent,
            idempotency_key,
        } => create_mailbox::handle_create_mailbox(engine, account, name, parent, idempotency_key),
        ActionRequest::RenameMailbox {
            mailbox,
            name,
            idempotency_key,
        } => rename_mailbox::handle_rename_mailbox(engine, mailbox, name, idempotency_key),
        ActionRequest::DeleteMailbox {
            mailbox,
            idempotency_key,
        } => delete_mailbox::handle_delete_mailbox(engine, mailbox, idempotency_key),
        ActionRequest::EmptyMailbox {
            mailbox,
            idempotency_key,
        } => empty_mailbox::handle_empty_mailbox(engine, mailbox, idempotency_key),
    }
}
