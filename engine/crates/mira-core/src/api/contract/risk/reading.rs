//! Niveau de risque des requêtes : comptes, boîtes, messages, fils, recherche et maintenance.

use mira_vocabulary::ToolRisk;

use crate::api::contract::{
    AccountRequest, MailboxRequest, MaintenanceRequest, MessageRequest, SearchRequest,
    ThreadRequest,
};

/// Niveau de risque des requêtes du domaine account.
pub(super) fn account_risk(request: &AccountRequest) -> ToolRisk {
    match request {
        AccountRequest::ListAccounts => ToolRisk::Read,
        AccountRequest::PrepareAccount { .. }
        | AccountRequest::TestConnection { .. }
        | AccountRequest::AddAccount { .. }
        | AccountRequest::UpdateAccount { .. }
        | AccountRequest::PauseAccount { .. }
        | AccountRequest::ResumeAccount { .. }
        | AccountRequest::DeleteAccount { .. } => ToolRisk::Sensitive,
    }
}

/// Niveau de risque des requêtes du domaine mailbox.
pub(super) fn mailbox_risk(request: &MailboxRequest) -> ToolRisk {
    match request {
        MailboxRequest::ListMailboxes { .. } => ToolRisk::Read,
        MailboxRequest::SetSyncedMailboxes { .. } => ToolRisk::Sensitive,
    }
}

/// Niveau de risque des requêtes du domaine message.
pub(super) fn message_risk(request: &MessageRequest) -> ToolRisk {
    match request {
        MessageRequest::ListMessages(..)
        | MessageRequest::GetMessage { .. }
        | MessageRequest::GetMessageBody { .. }
        | MessageRequest::GetRawSource { .. }
        | MessageRequest::ListAttachments { .. }
        | MessageRequest::GetSafeHtml { .. } => ToolRisk::Read,
        MessageRequest::FetchMessageBody { .. } => ToolRisk::Reversible,
        MessageRequest::SaveAttachment(..) => ToolRisk::Confirmed,
        MessageRequest::AllowRemoteContent { .. } => ToolRisk::Sensitive,
    }
}

/// Niveau de risque des requêtes du domaine thread.
pub(super) fn thread_risk(request: &ThreadRequest) -> ToolRisk {
    match request {
        ThreadRequest::ListThreads { .. } | ThreadRequest::GetThread { .. } => ToolRisk::Read,
    }
}

/// Niveau de risque des requêtes du domaine search.
pub(super) fn search_risk(request: &SearchRequest) -> ToolRisk {
    match request {
        SearchRequest::Search { .. } | SearchRequest::GetServerSearchResult { .. } => {
            ToolRisk::Read
        }
        SearchRequest::SearchServer { .. } => ToolRisk::Reversible,
    }
}

/// Niveau de risque des requêtes du domaine maintenance.
pub(super) fn maintenance_risk(request: &MaintenanceRequest) -> ToolRisk {
    match request {
        MaintenanceRequest::GetStorageStats | MaintenanceRequest::CheckIntegrity => ToolRisk::Read,
        MaintenanceRequest::PurgeAttachmentCache | MaintenanceRequest::RebuildSearchIndex => {
            ToolRisk::Reversible
        }
        MaintenanceRequest::ExportDiagnostic { .. } | MaintenanceRequest::ExportProfile { .. } => {
            ToolRisk::Sensitive
        }
    }
}
