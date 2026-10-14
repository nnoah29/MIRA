//! Noms des variantes des domaines de lecture : le `match` sans `_` casse la compilation si une variante naît.

use mira_assembly::api::{
    MailboxRequest, MaintenanceRequest, MessageRequest, SearchRequest, ThreadRequest,
};

/// Nom d'une requête du domaine boîtes.
pub fn mailbox_name(request: &MailboxRequest) -> &'static str {
    match request {
        MailboxRequest::ListMailboxes { .. } => "mailbox.list_mailboxes",
        MailboxRequest::SetSyncedMailboxes { .. } => "mailbox.set_synced_mailboxes",
    }
}

/// Nom d'une requête du domaine lecture des messages.
pub fn message_name(request: &MessageRequest) -> &'static str {
    match request {
        MessageRequest::ListMessages(_) => "message.list_messages",
        MessageRequest::GetMessage { .. } => "message.get_message",
        MessageRequest::GetMessageBody { .. } => "message.get_message_body",
        MessageRequest::FetchMessageBody { .. } => "message.fetch_message_body",
        MessageRequest::GetRawSource { .. } => "message.get_raw_source",
        MessageRequest::ListAttachments { .. } => "message.list_attachments",
        MessageRequest::SaveAttachment(_) => "message.save_attachment",
        MessageRequest::GetSafeHtml { .. } => "message.get_safe_html",
        MessageRequest::AllowRemoteContent { .. } => "message.allow_remote_content",
    }
}

/// Nom d'une requête du domaine fils.
pub fn thread_name(request: &ThreadRequest) -> &'static str {
    match request {
        ThreadRequest::ListThreads { .. } => "thread.list_threads",
        ThreadRequest::GetThread { .. } => "thread.get_thread",
    }
}

/// Nom d'une requête du domaine recherche.
pub fn search_name(request: &SearchRequest) -> &'static str {
    match request {
        SearchRequest::Search { .. } => "search.search",
        SearchRequest::SearchServer { .. } => "search.search_server",
        SearchRequest::GetServerSearchResult { .. } => "search.get_server_search_result",
    }
}

/// Nom d'une requête du domaine maintenance.
pub fn maintenance_name(request: &MaintenanceRequest) -> &'static str {
    match request {
        MaintenanceRequest::GetStorageStats => "maintenance.get_storage_stats",
        MaintenanceRequest::PurgeAttachmentCache => "maintenance.purge_attachment_cache",
        MaintenanceRequest::RebuildSearchIndex => "maintenance.rebuild_search_index",
        MaintenanceRequest::CheckIntegrity => "maintenance.check_integrity",
        MaintenanceRequest::ExportDiagnostic { .. } => "maintenance.export_diagnostic",
        MaintenanceRequest::ExportProfile { .. } => "maintenance.export_profile",
    }
}
