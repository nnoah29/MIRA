#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Envoi interrompu (décision 9, inv. 26) : incertain, jamais renvoyé seul, vérifié dans Envoyés, décidé par l'utilisateur.

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::{SEND_UNDO_DELAY_MILLIS, recover_interrupted_actions};
use mira_core::send::resolve_uncertain_send;
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{ActionId, ActionState, DraftState, EventKind, ProviderQuirks};
use support_actions::drafts::stored;
use support_actions::inspect::force_state;
use support_imap::mail::mail;
use support_imap::within;
use support_replay::fixture::Fixture;
use support_replay::queue_view::state_of;
use support_replay::sends::{received_texts, schedule_ready_draft, stored_message_id};

/// Envoi dont la connexion SMTP est coupée après DATA : le message est parti mais le moteur l'ignore
async fn interrupted_send() -> (Fixture, mira_vocabulary::Draft, ActionId) {
    let fx = Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await;
    let (draft, action) = schedule_ready_draft(&fx, "k-26");
    fx.smtp.drop_after_data_once();
    fx.advance(SEND_UNDO_DELAY_MILLIS);
    let mut client = fx.client().await;
    let report = fx.replay(&mut client).await;
    assert_eq!((report.uncertain, report.succeeded), (1, 0));
    (fx, draft, action)
}

/// Inv. 26 : la coupure après DATA rend l'envoi incertain ; ni le rejeu ni la reprise ne le renvoient
#[tokio::test]
async fn invariant_26_interrupted_send_becomes_uncertain_never_resent() {
    within(async {
        let (fx, draft, action) = interrupted_send().await;
        assert_eq!(state_of(&fx, &action), ActionState::Uncertain);
        assert_eq!(stored(&fx.world, &draft).state, DraftState::Sending);
        let uncertain = |kind: &EventKind| matches!(kind, EventKind::SendUncertain { .. });
        assert_eq!(
            fx.world
                .sink
                .kinds()
                .iter()
                .filter(|k| uncertain(k))
                .count(),
            1
        );
        fx.advance(3_600_000);
        let mut restarted = fx.client().await;
        let recovered = recover_interrupted_actions(&fx.ctx(), &mut restarted).await;
        assert_eq!(recovered.unwrap().sends_uncertain, 0);
        assert_eq!(fx.replay(&mut restarted).await.succeeded, 0);
        assert_eq!(fx.smtp.received().len(), 1);
        assert_eq!(state_of(&fx, &action), ActionState::Uncertain);
    })
    .await;
}

/// Redémarrage avec un envoi « en cours » absent d'Envoyés : incertain, événement émis, aucun renvoi
#[tokio::test]
async fn restart_with_send_in_progress_and_no_trace_in_sent_is_uncertain() {
    within(async {
        let (fx, _, action) = interrupted_send().await;
        force_state(&fx.world, &action, ActionState::InProgress);
        let mut client = fx.client().await;
        let report = recover_interrupted_actions(&fx.ctx(), &mut client)
            .await
            .unwrap();
        assert_eq!((report.sends_uncertain, report.sends_confirmed), (1, 0));
        assert_eq!(state_of(&fx, &action), ActionState::Uncertain);
        assert_eq!(fx.smtp.received().len(), 1);
    })
    .await;
}

/// Redémarrage avec un envoi « en cours » dont le message est dans Envoyés : réussi, brouillon supprimé
#[tokio::test]
async fn restart_finds_the_message_in_sent_and_confirms_the_send() {
    within(async {
        let (fx, draft, action) = interrupted_send().await;
        force_state(&fx.world, &action, ActionState::InProgress);
        let id = stored_message_id(&fx, &action).unwrap();
        let copy = mail("Copie", id.trim_matches(['<', '>']));
        fx.env.store.deliver("alice", "Sent", &copy, &["\\Seen"]);
        let mut client = fx.client().await;
        let report = recover_interrupted_actions(&fx.ctx(), &mut client)
            .await
            .unwrap();
        assert_eq!(report.sends_confirmed, 1);
        assert_eq!(state_of(&fx, &action), ActionState::Succeeded);
        assert!(
            fx.world
                .db
                .read(|tx| tx.get_draft(&draft.id))
                .unwrap()
                .is_none()
        );
        assert_eq!(fx.smtp.received().len(), 1);
    })
    .await;
}

/// L'utilisateur choisit de renvoyer : nouvelle tentative avec un nouveau `Message-ID`
#[tokio::test]
async fn resolving_an_uncertain_send_with_resend_sends_again_with_a_new_message_id() {
    within(async {
        let (fx, draft, action) = interrupted_send().await;
        let first = stored_message_id(&fx, &action).unwrap();
        resolve_uncertain_send(&fx.world.ctx(), &action, true).unwrap();
        assert_eq!(state_of(&fx, &action), ActionState::Pending);
        let mut client = fx.client().await;
        assert_eq!(fx.replay(&mut client).await.succeeded, 1);
        assert_eq!(fx.smtp.received().len(), 2);
        let second = stored_message_id(&fx, &action).unwrap();
        assert_ne!(first, second);
        assert!(received_texts(&fx)[1].contains(&format!("Message-ID: {second}")));
        assert!(
            fx.world
                .db
                .read(|tx| tx.get_draft(&draft.id))
                .unwrap()
                .is_none()
        );
    })
    .await;
}

/// L'utilisateur abandonne : l'action est annulée et le brouillon revient en édition, rien n'est renvoyé
#[tokio::test]
async fn resolving_an_uncertain_send_with_abandon_returns_the_draft_to_editing() {
    within(async {
        let (fx, draft, action) = interrupted_send().await;
        resolve_uncertain_send(&fx.world.ctx(), &action, false).unwrap();
        assert_eq!(state_of(&fx, &action), ActionState::Cancelled);
        assert_eq!(stored(&fx.world, &draft).state, DraftState::Editing);
        let mut client = fx.client().await;
        assert_eq!(fx.replay(&mut client).await.succeeded, 0);
        assert_eq!(fx.smtp.received().len(), 1);
        let error = resolve_uncertain_send(&fx.world.ctx(), &action, true).unwrap_err();
        assert_eq!(error.code, "incompatible_state.action_not_retryable");
    })
    .await;
}
