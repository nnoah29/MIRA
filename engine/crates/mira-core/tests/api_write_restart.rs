#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Reprise après redémarrage du moteur par la façade : envoi incertain jamais renvoyé, décision de l'utilisateur (inv. 18, 26).

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_write;

use mira_core::api::{ComposeRequest, Request};
use mira_vocabulary::{ActionId, ActionKind, ActionState, DraftState};
use support_imap::within;
use support_write::builders::resolve_request;
use support_write::fixture::Fixture;
use support_write::timing::settle;
use support_write::waits::eventually;

/// Inv. 26 : au redémarrage, l'envoi « en cours » absent d'Envoyés devient incertain, événement émis, aucun renvoi
#[tokio::test]
async fn invariant_26_restart_makes_interrupted_send_uncertain_never_resent() {
    within(async {
        let mut f = Fixture::start().await;
        let (draft, action) = f.interrupted_send().await;
        assert_eq!(f.uncertain_events().await, 1);
        f.crash_in_progress(&action).await;
        f.wait_uncertain_events(2).await;
        assert_eq!(f.uncertain_events().await, 2);
        f.advance(3_600_000);
        settle().await;
        assert_eq!(f.smtp.received().len(), 1);
        assert_eq!(f.state_of(&action), ActionState::Uncertain);
        assert_eq!(f.get_draft(&draft.id).await.state, DraftState::Sending);
    })
    .await;
}

/// Renvoyer après décision : nouveau Message-ID, un second message part, l'ancien reste compté
#[tokio::test]
async fn resolve_uncertain_send_resend_uses_a_new_message_id() {
    within(async {
        let f = Fixture::start().await;
        let (_, action) = f.interrupted_send().await;
        f.run(resolve_request(&action, true)).await;
        f.wait_for_state(&action, ActionState::Succeeded).await;
        let ids = f.smtp_message_ids();
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
    })
    .await;
}

/// Abandonner après décision : action annulée, brouillon de nouveau modifiable, rien n'est renvoyé
#[tokio::test]
async fn resolve_uncertain_send_abandon_returns_the_draft_to_editing() {
    within(async {
        let f = Fixture::start().await;
        let (draft, action) = f.interrupted_send().await;
        f.run(resolve_request(&action, false)).await;
        assert_eq!(f.state_of(&action), ActionState::Cancelled);
        assert_eq!(f.get_draft(&draft.id).await.state, DraftState::Editing);
        f.advance(3_600_000);
        settle().await;
        assert_eq!(f.smtp.received().len(), 1);
    })
    .await;
}

/// Redémarrage avec un envoi « en cours » dont le message est déjà dans Envoyés : réussi, brouillon supprimé
#[tokio::test]
async fn restart_confirms_a_send_found_in_the_sent_folder() {
    within(async {
        let mut f = Fixture::start().await;
        let (draft, action) = f.interrupted_send().await;
        let raw = f.smtp.received()[0].data.clone();
        f.store.deliver(support_write::USER, "Sent", &raw, &[]);
        f.crash_in_progress(&action).await;
        eventually("envoi confirmé", || {
            f.state_of(&action) == ActionState::Succeeded
        })
        .await;
        assert_eq!(f.smtp.received().len(), 1);
        let missing = ComposeRequest::GetDraft { draft: draft.id };
        assert_eq!(
            f.refused(Request::Compose(missing)).await.code,
            "not_found.draft"
        );
    })
    .await;
}

/// Redémarrage avec une action ordinaire « en cours » : rejouée une fois, un seul effet sur le serveur
#[tokio::test]
async fn restart_replays_an_interrupted_flag_action_with_a_single_effect() {
    within(async {
        let mut f = Fixture::start().await;
        let ids = f
            .apply(ActionKind::MarkRead { read: true }, &["msg-1"], "k1")
            .await;
        f.advance(5_001);
        f.wait_for_state(&ids[0], ActionState::Succeeded).await;
        f.crash_leaving(&ids[0], ActionState::InProgress);
        eventually("action rejouée", || {
            f.state_of(&ids[0]) == ActionState::Succeeded
        })
        .await;
        let seen = |flag: &&String| flag.eq_ignore_ascii_case("\\Seen");
        assert_eq!(f.server_flags("INBOX", 1).iter().filter(seen).count(), 1);
    })
    .await;
}

/// Seul un envoi incertain se résout : toute autre action est refusée
#[tokio::test]
async fn resolve_uncertain_send_refuses_other_actions() {
    within(async {
        let f = Fixture::start().await;
        let ids = f
            .apply(ActionKind::MarkRead { read: true }, &["msg-1"], "k1")
            .await;
        let error = f.refused(resolve_request(&ids[0], true)).await;
        assert_eq!(error.code, "incompatible_state.action_not_retryable");
        let unknown = resolve_request(&ActionId::from_raw("action-inconnue"), false);
        assert_eq!(f.refused(unknown).await.code, "not_found.action");
    })
    .await;
}
