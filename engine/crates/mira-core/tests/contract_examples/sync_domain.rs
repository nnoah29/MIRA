//! Exemples du domaine « synchronisation » : lancer, pause, état, remonter la plage, resynchroniser.

use mira_core::api::{AccountSyncStatus, Request, Response, SyncRequest, SyncResponse, SyncScope};
use mira_vocabulary::{AccountId, AccountState, MailboxId, Timestamp};

use super::{Named, numbered};

/// Une requête par forme de `SyncRequest` (trois portées pour `StartSync`)
pub fn requests() -> Vec<Named<Request>> {
    let account = AccountId::from_raw("account-1");
    let mailbox = MailboxId::from_raw("mailbox-1");
    let items = vec![
        SyncRequest::StartSync {
            scope: SyncScope::All,
        },
        SyncRequest::StartSync {
            scope: SyncScope::Account {
                account: account.clone(),
            },
        },
        SyncRequest::StartSync {
            scope: SyncScope::Mailbox {
                mailbox: mailbox.clone(),
            },
        },
        SyncRequest::PauseSync { account },
        SyncRequest::GetSyncStatus,
        SyncRequest::ExtendWindow {
            mailbox: mailbox.clone(),
            days: 90,
        },
        SyncRequest::ResyncMailbox { mailbox },
    ];
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    let names = numbered("sync", &ops);
    let wrap = |request| Request::Sync(request);
    names.into_iter().zip(items.into_iter().map(wrap)).collect()
}

/// Exemple d'état de synchronisation d'un compte
fn status_sample() -> AccountSyncStatus {
    AccountSyncStatus {
        account: AccountId::from_raw("account-1"),
        state: AccountState::Connected,
        syncing: true,
        last_synced_at: Some(Timestamp::from_millis(1_700_000_000_000)),
        last_error: None,
        mailboxes_in_progress: vec![MailboxId::from_raw("mailbox-1")],
        pending_actions: 2,
    }
}

/// Une réponse par forme de `SyncResponse`
pub fn responses() -> Vec<Named<Response>> {
    let status = Response::Sync(SyncResponse::Status(vec![status_sample()]));
    let paused = Response::Sync(SyncResponse::Paused(status_sample()));
    vec![
        ("sync.status".to_string(), status),
        ("sync.paused".to_string(), paused),
    ]
}

/// Nom de l'opération, par `match` exhaustif
pub fn op_name(request: &SyncRequest) -> &'static str {
    match request {
        SyncRequest::StartSync { .. } => "start_sync",
        SyncRequest::PauseSync { .. } => "pause_sync",
        SyncRequest::GetSyncStatus => "get_sync_status",
        SyncRequest::ExtendWindow { .. } => "extend_window",
        SyncRequest::ResyncMailbox { .. } => "resync_mailbox",
    }
}
