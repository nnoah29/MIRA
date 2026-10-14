#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Reprise d'un envoi retrouvé dans Envoyés : la copie serveur du brouillon est purgée sans faire échouer la reprise.

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::{SEND_UNDO_DELAY_MILLIS, recover_interrupted_actions};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{ActionId, ActionState, Draft, ProviderQuirks};
use support_actions::inspect::force_state;
use support_imap::mail::mail;
use support_imap::within;
use support_replay::fixture::Fixture;
use support_replay::queue_view::state_of;
use support_replay::sends::{schedule_ready_draft, stored_message_id};

/// Envoi coupé après DATA, repéré dans Envoyés et dont le brouillon a une copie sur le serveur
async fn interrupted_send_found_in_sent() -> (Fixture, Draft, ActionId) {
    let fx = Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await;
    let (draft, action) = schedule_ready_draft(&fx, "k-copy");
    fx.smtp.drop_after_data_once();
    fx.advance(SEND_UNDO_DELAY_MILLIS);
    let mut client = fx.client().await;
    assert_eq!(fx.replay(&mut client).await.uncertain, 1);
    force_state(&fx.world, &action, ActionState::InProgress);
    let sent_id = stored_message_id(&fx, &action).unwrap();
    let sent = mail("Copie", sent_id.trim_matches(['<', '>']));
    fx.env.store.deliver("alice", "Sent", &sent, &["\\Seen"]);
    let copy = mail("Brouillon", &format!("draft-{}@mira.local", draft.id));
    fx.env.store.deliver("alice", "Drafts", &copy, &["\\Draft"]);
    (fx, draft, action)
}

/// Un envoi retrouvé dans Envoyés fait disparaître la copie serveur du brouillon
#[tokio::test]
async fn regression_recovered_send_purges_the_server_draft_copy() {
    within(async {
        let (fx, _, action) = interrupted_send_found_in_sent().await;
        let mut client = fx.client().await;
        let report = recover_interrupted_actions(&fx.ctx(), &mut client)
            .await
            .unwrap();
        assert_eq!(report.sends_confirmed, 1);
        assert_eq!(state_of(&fx, &action), ActionState::Succeeded);
        assert!(fx.env.store.messages("alice", "Drafts").is_empty());
    })
    .await;
}

/// Une purge refusée par le serveur est seulement journalisée : l'envoi reste réussi
#[tokio::test]
async fn regression_failed_draft_copy_purge_does_not_fail_the_recovery() {
    within(async {
        let (fx, draft, action) = interrupted_send_found_in_sent().await;
        let mut client = fx.client().await;
        fx.env.store.fail_next("UID STORE", "NO [SERVERBUG] refus");
        let report = recover_interrupted_actions(&fx.ctx(), &mut client)
            .await
            .unwrap();
        assert_eq!(report.sends_confirmed, 1);
        assert_eq!(state_of(&fx, &action), ActionState::Succeeded);
        assert_eq!(fx.env.store.messages("alice", "Drafts").len(), 1);
        let gone = fx.world.db.read(|tx| tx.get_draft(&draft.id)).unwrap();
        assert!(gone.is_none());
    })
    .await;
}
