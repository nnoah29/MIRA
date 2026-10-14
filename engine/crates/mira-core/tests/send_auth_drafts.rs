#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Authentification pendant l'envoi (S10) et copie du brouillon sur le serveur (SaveDraft).

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::{ReplayStop, SEND_UNDO_DELAY_MILLIS, UNDO_DELAY_MILLIS};
use mira_core::drafts::{DraftEdit, edit_draft};
use mira_core::imap::ImapClient;
use mira_vocabulary::{AccountState, ActionKind, ActionState, Draft, DraftState};
use support_actions::drafts::{ready_draft, stored};
use support_actions::requests::submit_folders;
use support_imap::within;
use support_replay::credentials::FakeCredentials;
use support_replay::fixture::Fixture;
use support_replay::observe::{server_flags, server_message_ids};
use support_replay::presets::standard as started;
use support_replay::queue_view::state_of;
use support_replay::sends::schedule_ready_draft;

/// Le serveur SMTP refuse les identifiants et ils ne se rafraîchissent pas : l'envoi reste en file, le compte se réauthentifie
#[tokio::test]
async fn smtp_authentication_failure_keeps_the_send_queued_and_asks_to_reauthenticate() {
    within(async {
        let mut fx = started().await;
        fx.credentials = FakeCredentials::rejected_password();
        let (draft, action) = schedule_ready_draft(&fx, "k-auth");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!(report.stopped, Some(ReplayStop::AuthRequired));
        assert_eq!(state_of(&fx, &action), ActionState::Pending);
        let state = stored(&fx.world, &draft).state;
        assert!(matches!(state, DraftState::Scheduled { .. }));
        let account = fx.world.db.read(|tx| tx.get_account(&fx.account)).unwrap();
        assert_eq!(account.unwrap().state, AccountState::AuthRequired);
        assert!(fx.smtp.received().is_empty());
    })
    .await;
}

/// Un refus SMTP d'authentification déclenche un rafraîchissement silencieux puis un seul nouvel essai qui réussit
#[tokio::test]
async fn smtp_authentication_refusal_triggers_silent_renewal_then_sends() {
    within(async {
        let mut fx = started().await;
        fx.credentials = FakeCredentials::stale_login();
        let (_, action) = schedule_ready_draft(&fx, "k-renew");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        assert_eq!(fx.replay(&mut client).await.succeeded, 1);
        assert_eq!(fx.credentials.renewals(), 1);
        assert_eq!(state_of(&fx, &action), ActionState::Succeeded);
        assert_eq!(fx.smtp.received().len(), 1);
    })
    .await;
}

/// Soumet l'enregistrement du brouillon sur le serveur puis le rejoue
async fn save_and_replay(fx: &Fixture, client: &mut ImapClient, draft: &Draft, key: &str) {
    let save = ActionKind::SaveDraft {
        draft: draft.id.clone(),
    };
    submit_folders(&fx.world, save, &[], key);
    fx.advance(UNDO_DELAY_MILLIS + 1);
    assert_eq!(fx.replay(client).await.succeeded, 1);
}

/// SaveDraft dépose le brouillon dans la boîte Brouillons, puis remplace sa copie à chaque enregistrement
#[tokio::test]
async fn saving_a_draft_replaces_its_server_copy() {
    within(async {
        let fx = started().await;
        let draft = ready_draft(&fx.world);
        let mut client = fx.client().await;
        save_and_replay(&fx, &mut client, &draft, "k-save-1").await;
        let wanted = format!("draft-{}@mira.local", draft.id);
        let ids = server_message_ids(&fx, "Drafts");
        assert_eq!(ids, std::slice::from_ref(&wanted));
        assert_eq!(server_flags(&fx, "Drafts", &wanted), ["\\Draft", "\\Seen"]);
        let edit = DraftEdit {
            subject: Some("Nouvel objet".to_string()),
            ..DraftEdit::default()
        };
        edit_draft(&fx.world.ctx(), &draft.id, edit).unwrap();
        save_and_replay(&fx, &mut client, &draft, "k-save-2").await;
        let ids = server_message_ids(&fx, "Drafts");
        assert_eq!(ids, std::slice::from_ref(&wanted));
    })
    .await;
}
