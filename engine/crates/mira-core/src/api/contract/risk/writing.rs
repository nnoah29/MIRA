//! Niveau de risque des requêtes : actions, rédaction, file d'actions et synchronisation.

use mira_vocabulary::ToolRisk;

use crate::api::contract::{ActionRequest, ComposeRequest, QueueRequest, SyncRequest};

/// Niveau de risque des requêtes du domaine action.
pub(super) fn action_risk(request: &ActionRequest) -> ToolRisk {
    match request {
        ActionRequest::UndoAction { .. } => ToolRisk::Reversible,
        ActionRequest::ApplyAction { .. }
        | ActionRequest::CreateMailbox { .. }
        | ActionRequest::RenameMailbox { .. }
        | ActionRequest::DeleteMailbox { .. }
        | ActionRequest::EmptyMailbox { .. } => ToolRisk::Confirmed,
    }
}

/// Niveau de risque des requêtes du domaine compose.
pub(super) fn compose_risk(request: &ComposeRequest) -> ToolRisk {
    match request {
        ComposeRequest::GetDraft { .. }
        | ComposeRequest::ListDrafts { .. }
        | ComposeRequest::ListOutbox { .. } => ToolRisk::Read,
        ComposeRequest::CreateDraft { .. }
        | ComposeRequest::UpdateDraft { .. }
        | ComposeRequest::AttachFile { .. }
        | ComposeRequest::RemoveAttachment { .. }
        | ComposeRequest::SaveDraftCopy { .. } => ToolRisk::Reversible,
        ComposeRequest::DeleteDraft { .. }
        | ComposeRequest::SendDraft { .. }
        | ComposeRequest::CancelSend { .. }
        | ComposeRequest::ResolveUncertainSend { .. } => ToolRisk::Confirmed,
    }
}

/// Niveau de risque des requêtes du domaine queue.
pub(super) fn queue_risk(request: &QueueRequest) -> ToolRisk {
    match request {
        QueueRequest::ListActions { .. } => ToolRisk::Read,
        QueueRequest::RetryAction { .. } | QueueRequest::CancelAction { .. } => {
            ToolRisk::Reversible
        }
    }
}

/// Niveau de risque des requêtes du domaine sync.
pub(super) fn sync_risk(request: &SyncRequest) -> ToolRisk {
    match request {
        SyncRequest::GetSyncStatus => ToolRisk::Read,
        SyncRequest::StartSync { .. }
        | SyncRequest::PauseSync { .. }
        | SyncRequest::ExtendWindow { .. }
        | SyncRequest::ResyncMailbox { .. } => ToolRisk::Reversible,
    }
}
