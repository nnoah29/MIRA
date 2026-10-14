//! Réponses du moteur sur l'état et la synchronisation.
#![allow(dead_code)]

use crate::sidebar_fixtures::account;
use mira_assembly::api::{AccountStatus, AccountSyncStatus, EngineStatus, Response, SyncResponse};
use mira_assembly::mira_vocabulary::{AccountId, AccountState, ProfileId};
use mira_tui::commands::Tag;
use mira_tui::update::{Msg, Reply};

/// Réponse de l'état du moteur avec ces nombres d'actions en file pour a1 puis a2.
pub fn status_reply(pending: &[u64]) -> Msg {
    let accounts = pending
        .iter()
        .enumerate()
        .map(|(n, count)| AccountStatus {
            account: account(&format!("a{}", n + 1)),
            pending_actions: *count,
        })
        .collect();
    Msg::Reply(Reply {
        tag: Tag::Status,
        result: Ok(Response::Status(EngineStatus {
            profile: ProfileId::from_raw("profile"),
            data_directory: "/tmp/p".to_string(),
            accounts,
        })),
    })
}

/// Réponse de l'état de synchronisation du compte a1.
pub fn sync_status_reply(syncing: bool, state: AccountState, error: Option<&str>) -> Msg {
    let status = AccountSyncStatus {
        account: AccountId::from_raw("a1"),
        state,
        syncing,
        last_synced_at: None,
        last_error: error.map(str::to_string),
        mailboxes_in_progress: Vec::new(),
        pending_actions: 0,
    };
    Msg::Reply(Reply {
        tag: Tag::SyncStatus,
        result: Ok(Response::Sync(SyncResponse::Status(vec![status]))),
    })
}
