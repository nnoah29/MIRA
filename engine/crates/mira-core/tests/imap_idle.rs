#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Réception en temps réel (IDLE, 10.6) contre le faux serveur.

mod support_imap;

use std::time::Duration;

use mira_core::imap::IdleOutcome;
use mira_testkit::imap::{Capability, CapabilitySet};
use support_imap::env::start;
use support_imap::logins::selected;
use support_imap::mail::MAIL;
use support_imap::within;

/// Une livraison pendant l'attente réveille le client
#[tokio::test]
async fn idle_wakes_up_on_delivery() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = selected(&env).await;
        let store = env.store.clone();
        let deliver = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(150)).await;
            store.deliver("alice", "INBOX", MAIL, &[]);
        });
        let outcome = client
            .idle_until_change(Duration::from_secs(5))
            .await
            .unwrap();
        deliver.await.unwrap();
        assert_eq!(outcome, IdleOutcome::Changed);
    })
    .await;
}

/// Sans changement, l'attente se termine au délai et la session reste utilisable
#[tokio::test]
async fn idle_times_out_and_session_stays_usable() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = selected(&env).await;
        let outcome = client
            .idle_until_change(Duration::from_millis(200))
            .await
            .unwrap();
        assert_eq!(outcome, IdleOutcome::TimedOut);
        assert!(client.all_uids().await.unwrap().is_empty());
    })
    .await;
}

/// Un serveur sans IDLE est signalé par une erreur d'extension absente
#[tokio::test]
async fn idle_without_extension_is_refused() {
    within(async {
        let env = start(CapabilitySet::full().without(Capability::Idle)).await;
        let mut client = selected(&env).await;
        let error = client
            .idle_until_change(Duration::from_millis(200))
            .await
            .unwrap_err();
        assert_eq!(error.code, "server.extension_missing");
    })
    .await;
}
