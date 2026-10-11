#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Cycle de vie des comptes par la façade : pause, reprise, modification, suppression (S14, 7.9).

mod support_api;

use mira_core::api::{AccountChanges, AccountRequest, Request, Response};
use mira_vocabulary::{AccountId, AccountState, AuthMethodKind, EventKind, SecretKind};
use support_api::fixture;
use support_api::queries::{accounts, event_kinds, identities, state_changes};
use support_api::requests::{account_response, add_account, draft};

/// Pause et reprise émettent `AccountStateChanged` ; répéter l'opération n'émet rien de plus
#[tokio::test]
async fn pause_and_resume_emit_account_state_changed() {
    let f = fixture::start().await;
    let id = f.add_folders_account("k").await;
    let paused = account_response(
        &f.engine,
        AccountRequest::PauseAccount {
            account: id.clone(),
        },
    )
    .await;
    assert_eq!(paused.state, AccountState::Paused);
    account_response(
        &f.engine,
        AccountRequest::PauseAccount {
            account: id.clone(),
        },
    )
    .await;
    let resumed = account_response(
        &f.engine,
        AccountRequest::ResumeAccount {
            account: id.clone(),
        },
    )
    .await;
    assert_eq!(resumed.state, AccountState::Offline);
    let states = state_changes(&f.engine).await;
    assert_eq!(states, vec![AccountState::Paused, AccountState::Offline]);
}

/// La modification valide ses champs, émet `AccountUpdated` et ne touche pas aux champs absents
#[tokio::test]
async fn update_account_changes_only_the_given_fields() {
    let f = fixture::start().await;
    let id = f.add_folders_account("k").await;
    let changes = AccountChanges {
        display_name: Some("  Nouveau nom ".to_string()),
        window_days: Some(30),
        ..AccountChanges::default()
    };
    let updated = account_response(
        &f.engine,
        AccountRequest::UpdateAccount {
            account: id.clone(),
            changes,
        },
    )
    .await;
    assert_eq!(updated.display_name, "Nouveau nom");
    assert_eq!(updated.sync.window_days, 30);
    assert_eq!(updated.sync.prefetch_bodies, 50);
    assert_eq!(accounts(&f.engine).await, vec![updated]);
    assert!(
        event_kinds(&f.engine)
            .await
            .contains(&EventKind::AccountUpdated { account: id })
    );
}

/// Un nom vide, une fenêtre nulle ou une boîte inconnue sont refusés sans rien modifier
#[tokio::test]
async fn update_account_refuses_invalid_changes() {
    let f = fixture::start().await;
    let id = f.add_folders_account("k").await;
    let blank = AccountChanges {
        display_name: Some("  ".into()),
        ..AccountChanges::default()
    };
    let zero = AccountChanges {
        window_days: Some(0),
        ..AccountChanges::default()
    };
    let foreign = AccountChanges {
        synced_mailboxes: Some(vec![mira_vocabulary::MailboxId::from_raw("nope")]),
        ..AccountChanges::default()
    };
    for changes in [blank, zero, foreign] {
        let request = Request::Account(AccountRequest::UpdateAccount {
            account: id.clone(),
            changes,
        });
        let error = f.engine.execute(request).await.unwrap_err();
        assert_eq!(error.code, "invalid_input.parameter");
    }
    assert_eq!(accounts(&f.engine).await[0].display_name, "Test");
}

/// S14 : la suppression efface le compte et ses secrets, laisse les autres comptes et le serveur intacts
#[tokio::test]
async fn scenario_s14_delete_account_keeps_others_and_server() {
    let f = fixture::start().await;
    let doomed = f.add_folders_account("k1").await;
    let labels = draft(fixture::LABELS_USER, AuthMethodKind::AppPassword);
    let kept = add_account(&f.engine, labels, fixture::LABELS_PASSWORD, "k2")
        .await
        .id;
    let commands_before = f.store.command_log();
    let connections_before = f.imap.connection_count();
    let request = Request::Account(AccountRequest::DeleteAccount {
        account: doomed.clone(),
    });
    assert_eq!(f.engine.execute(request).await.unwrap(), Response::Done);
    let remaining: Vec<AccountId> = accounts(&f.engine)
        .await
        .into_iter()
        .map(|a| a.id)
        .collect();
    assert_eq!(remaining, vec![kept.clone()]);
    assert!(f.stored_secret(&doomed, SecretKind::Password).is_none());
    assert!(f.stored_secret(&kept, SecretKind::Password).is_some());
    assert_eq!(identities(&f.engine, &kept).await.len(), 1);
    assert_eq!(f.store.command_log(), commands_before);
    assert_eq!(f.imap.connection_count(), connections_before);
    assert!(
        event_kinds(&f.engine)
            .await
            .contains(&EventKind::AccountDeleted { account: doomed })
    );
}

/// Supprimer ou modifier un compte inconnu donne `not_found.account`
#[tokio::test]
async fn operations_on_unknown_account_are_not_found() {
    let f = fixture::start().await;
    let ghost = AccountId::from_raw("ghost");
    let requests = [
        AccountRequest::DeleteAccount {
            account: ghost.clone(),
        },
        AccountRequest::PauseAccount {
            account: ghost.clone(),
        },
        AccountRequest::ResumeAccount { account: ghost },
    ];
    for request in requests {
        let error = f
            .engine
            .execute(Request::Account(request))
            .await
            .unwrap_err();
        assert_eq!(error.code, "not_found.account");
    }
}
