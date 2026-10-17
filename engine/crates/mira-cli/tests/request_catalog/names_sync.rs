//! Noms des variantes du domaine synchronisation : le `match` sans `_` casse la compilation si une variante naît.

use mira_assembly::api::SyncRequest;

/// Nom d'une requête du domaine synchronisation.
pub fn sync_name(request: &SyncRequest) -> &'static str {
    match request {
        SyncRequest::StartSync { .. } => "sync.start_sync",
        SyncRequest::PauseSync { .. } => "sync.pause_sync",
        SyncRequest::GetSyncStatus => "sync.get_sync_status",
        SyncRequest::ExtendWindow { .. } => "sync.extend_window",
        SyncRequest::ResyncMailbox { .. } => "sync.resync_mailbox",
    }
}
