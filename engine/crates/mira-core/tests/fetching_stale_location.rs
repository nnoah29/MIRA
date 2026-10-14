#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Téléchargement à la demande avec un UID périmé : jamais le message d'un autre n'est rangé sous l'identifiant du premier.

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use mira_core::api::{MessageRequest, Request};
use mira_core::operations::OperationStatus;
use mira_vocabulary::{MessageId, SyncMarker};
use support_read::calls::run;
use support_read::mail;
use support_read::world::{World, start_world};

/// Statut final du téléchargement du corps, et dit si une source a été rangée
async fn fetch_and_inspect(world: &World, message: &MessageId) -> (OperationStatus, bool) {
    let status = world.fetch_body(message).await;
    let request = Request::Message(MessageRequest::GetRawSource {
        message: message.clone(),
    });
    let stored = run(&world.fixture.engine, request).await.is_ok();
    (status, stored)
}

/// Un UID réattribué à un autre message (même numéro, autre Message-ID) n'est jamais rangé
#[tokio::test]
async fn regression_fetch_refuses_a_source_with_another_message_id() {
    let world = start_world().await;
    let id = world.deliver_header_only(&mail::plain("licorne", "le message attendu"));
    let (store, user) = (&world.fixture.store, world.user());
    store.expunge(user, "INBOX", 1);
    store.deliver(
        user,
        "INBOX",
        &mail::plain("narval", "un autre message"),
        &[],
    );
    store.renumber(user, "INBOX");
    let (status, stored) = fetch_and_inspect(&world, &id).await;
    assert_eq!(
        status,
        OperationStatus::Failed {
            code: "not_found.message".to_string()
        }
    );
    assert!(!stored, "la source d'un autre message a été rangée");
}

/// Un UIDVALIDITY différent de celui du miroir interdit de se fier à l'UID, même si le Message-ID concorde
#[tokio::test]
async fn regression_fetch_refuses_a_changed_uid_validity() {
    let world = start_world().await;
    let id = world.deliver_header_only(&mail::plain("licorne", "le message attendu"));
    let marker = SyncMarker {
        uid_validity: 4_000_000_000,
        uid_next: 2,
        highest_modseq: None,
    };
    world
        .mirror
        .write(|tx| tx.set_mailbox_marker(&world.mailbox, Some(marker)))
        .unwrap();
    let (status, stored) = fetch_and_inspect(&world, &id).await;
    assert_eq!(
        status,
        OperationStatus::Failed {
            code: "not_found.message".to_string()
        }
    );
    assert!(!stored);
}

/// Une apparition en attente de revalidation n'a pas d'UID fiable : elle est ignorée
#[tokio::test]
async fn regression_fetch_ignores_an_appearance_awaiting_revalidation() {
    let world = start_world().await;
    let id = world.deliver_header_only(&mail::plain("licorne", "le message attendu"));
    world
        .mirror
        .write(|tx| tx.mark_mailbox_for_revalidation(&world.mailbox))
        .unwrap();
    let (status, stored) = fetch_and_inspect(&world, &id).await;
    assert_eq!(
        status,
        OperationStatus::Failed {
            code: "not_found.message".to_string()
        }
    );
    assert!(!stored);
}

/// Le cas normal reste servi : UID, validité et Message-ID concordent
#[tokio::test]
async fn fetch_still_succeeds_when_location_and_message_id_agree() {
    let world = start_world().await;
    let id = world.deliver_header_only(&mail::plain("licorne", "le message attendu"));
    let (status, stored) = fetch_and_inspect(&world, &id).await;
    assert_eq!(status, OperationStatus::Finished);
    assert!(stored);
}
