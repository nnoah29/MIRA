//! Noms des variantes des domaines d'écriture : le `match` sans `_` casse la compilation si une variante naît.

use mira_assembly::api::{ActionRequest, ComposeRequest, QueueRequest};

/// Nom d'une requête du domaine actions.
pub fn action_name(request: &ActionRequest) -> &'static str {
    match request {
        ActionRequest::ApplyAction { .. } => "action.apply_action",
        ActionRequest::UndoAction { .. } => "action.undo_action",
        ActionRequest::CreateMailbox { .. } => "action.create_mailbox",
        ActionRequest::RenameMailbox { .. } => "action.rename_mailbox",
        ActionRequest::DeleteMailbox { .. } => "action.delete_mailbox",
        ActionRequest::EmptyMailbox { .. } => "action.empty_mailbox",
    }
}

/// Nom d'une requête du domaine rédaction.
pub fn compose_name(request: &ComposeRequest) -> &'static str {
    match request {
        ComposeRequest::CreateDraft { .. } => "compose.create_draft",
        ComposeRequest::UpdateDraft { .. } => "compose.update_draft",
        ComposeRequest::AttachFile { .. } => "compose.attach_file",
        ComposeRequest::RemoveAttachment { .. } => "compose.remove_attachment",
        ComposeRequest::DeleteDraft { .. } => "compose.delete_draft",
        ComposeRequest::SaveDraftCopy { .. } => "compose.save_draft_copy",
        ComposeRequest::GetDraft { .. } => "compose.get_draft",
        ComposeRequest::ListDrafts { .. } => "compose.list_drafts",
        ComposeRequest::SendDraft { .. } => "compose.send_draft",
        ComposeRequest::CancelSend { .. } => "compose.cancel_send",
        ComposeRequest::ListOutbox { .. } => "compose.list_outbox",
        ComposeRequest::ResolveUncertainSend { .. } => "compose.resolve_uncertain_send",
    }
}

/// Nom d'une requête du domaine file d'actions.
pub fn queue_name(request: &QueueRequest) -> &'static str {
    match request {
        QueueRequest::ListActions { .. } => "queue.list_actions",
        QueueRequest::RetryAction { .. } => "queue.retry_action",
        QueueRequest::CancelAction { .. } => "queue.cancel_action",
    }
}
