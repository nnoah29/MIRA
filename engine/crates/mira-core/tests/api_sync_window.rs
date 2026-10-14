#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! « Remonter plus loin » (S12) et resynchronisation d'une boîte (S13) demandées par la façade.

mod support_api;

use mira_core::api::{OperationRequest, Request, SyncRequest};
use mira_core::operations::OperationStatus;
use mira_vocabulary::{EventKind, MailboxId};
use support_api::queries::event_kinds;
use support_api::sync_calls::{operation_outcome, start_operation};
use support_api::sync_control::{pause_sync_of, window_days};
use support_api::sync_events::count_kinds;
use support_api::sync_reads::{ids_in, mailbox_named, subjects_in};
use support_api::synced::aged_account;

/// S12 : remonter de 30 jours télécharge les en-têtes plus anciens, avec progression, et élargit la plage
#[tokio::test]
async fn scenario_s12_extend_window_through_facade_downloads_older_headers() {
    let (f, account) = aged_account().await;
    assert_eq!(subjects_in(&f.engine, &account, "INBOX").await.len(), 2);
    let mailbox = mailbox_named(&f.engine, &account, "INBOX").await.id;
    let info = start_operation(&f.engine, SyncRequest::ExtendWindow { mailbox, days: 30 }).await;
    assert_eq!(info.kind, "extend_window");
    let outcome = operation_outcome(&f.engine, &info).await;
    assert_eq!(outcome, OperationStatus::Finished);
    let found = subjects_in(&f.engine, &account, "INBOX").await;
    assert_eq!(found.len(), 5, "{found:?}");
    assert!(!found.contains(&"Très ancien".to_string()));
    assert_eq!(window_days(&f, &account).await, 120);
    let kinds = event_kinds(&f.engine).await;
    let full = |k: &EventKind| {
        matches!(
            k,
            EventKind::OperationProgress {
                done: 3,
                total: 3,
                ..
            }
        )
    };
    assert_eq!(count_kinds(&kinds, full), 1);
}

/// Une extension annulée avant son premier lot ne télécharge rien et laisse la plage inchangée
#[tokio::test]
async fn cancelled_extension_keeps_window_and_messages() {
    let (f, account) = aged_account().await;
    let mailbox = mailbox_named(&f.engine, &account, "INBOX").await.id;
    let info = start_operation(&f.engine, SyncRequest::ExtendWindow { mailbox, days: 30 }).await;
    let operation = info.id.clone();
    let cancel = Request::Operation(OperationRequest::CancelOperation { operation });
    f.engine.execute(cancel).await.unwrap();
    let outcome = operation_outcome(&f.engine, &info).await;
    assert_eq!(outcome, OperationStatus::Cancelled);
    assert_eq!(subjects_in(&f.engine, &account, "INBOX").await.len(), 2);
    assert_eq!(window_days(&f, &account).await, 90);
}

/// Zéro jour et boîte inconnue sont refusés sans rien lancer
#[tokio::test]
async fn extend_window_and_resync_refuse_invalid_targets() {
    let (f, account) = aged_account().await;
    let inbox = mailbox_named(&f.engine, &account, "INBOX").await.id;
    let ghost = MailboxId::from_raw("fantôme");
    let calls = [
        (
            SyncRequest::ExtendWindow {
                mailbox: inbox,
                days: 0,
            },
            "invalid_input.parameter",
        ),
        (
            SyncRequest::ExtendWindow {
                mailbox: ghost.clone(),
                days: 5,
            },
            "not_found.mailbox",
        ),
        (
            SyncRequest::ResyncMailbox { mailbox: ghost },
            "not_found.mailbox",
        ),
    ];
    for (request, code) in calls {
        let refused = f.engine.execute(Request::Sync(request)).await;
        assert_eq!(refused.unwrap_err().code, code);
    }
}

/// Un compte en pause refuse de remonter la plage et de resynchroniser
#[tokio::test]
async fn extend_window_and_resync_refuse_paused_accounts() {
    let (f, account) = aged_account().await;
    let mailbox = mailbox_named(&f.engine, &account, "INBOX").await.id;
    pause_sync_of(&f, &account).await;
    let extend = SyncRequest::ExtendWindow {
        mailbox: mailbox.clone(),
        days: 5,
    };
    for request in [extend, SyncRequest::ResyncMailbox { mailbox }] {
        let refused = f.engine.execute(Request::Sync(request)).await;
        let code = "incompatible_state.account_paused";
        assert_eq!(refused.unwrap_err().code, code);
    }
}

/// S13 : la resynchronisation forcée relit la plage, garde les identifiants internes et annonce `MailboxResynchronized`
#[tokio::test]
async fn scenario_s13_resync_mailbox_through_facade_keeps_internal_ids() {
    let (f, account) = aged_account().await;
    let before = ids_in(&f.engine, &account, "INBOX").await;
    let mailbox = mailbox_named(&f.engine, &account, "INBOX").await.id;
    let info = start_operation(&f.engine, SyncRequest::ResyncMailbox { mailbox }).await;
    assert_eq!(info.kind, "resync_mailbox");
    let outcome = operation_outcome(&f.engine, &info).await;
    assert_eq!(outcome, OperationStatus::Finished);
    assert_eq!(ids_in(&f.engine, &account, "INBOX").await, before);
    let kinds = event_kinds(&f.engine).await;
    let resynced = |k: &EventKind| matches!(k, EventKind::MailboxResynchronized { .. });
    assert_eq!(count_kinds(&kinds, resynced), 1);
}

/// S13 : le serveur renumérote la boîte, la resynchronisation retrouve les mêmes messages sous leurs identifiants
#[tokio::test]
async fn resync_after_server_renumbering_keeps_the_same_messages() {
    let (f, account) = aged_account().await;
    let before = ids_in(&f.engine, &account, "INBOX").await;
    f.store
        .renumber(support_api::fixture::FOLDERS_USER, "INBOX");
    let mailbox = mailbox_named(&f.engine, &account, "INBOX").await.id;
    let info = start_operation(&f.engine, SyncRequest::ResyncMailbox { mailbox }).await;
    let outcome = operation_outcome(&f.engine, &info).await;
    assert_eq!(outcome, OperationStatus::Finished);
    assert_eq!(ids_in(&f.engine, &account, "INBOX").await, before);
    assert_eq!(subjects_in(&f.engine, &account, "INBOX").await.len(), 2);
}
