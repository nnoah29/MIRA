#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Téléchargement explicite du corps contre le faux serveur IMAP : rangement, pièces jointes, annulation.

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use mira_core::api::{MessageRequest, OperationRequest, Request, Response};
use mira_core::operations::OperationStatus;
use mira_vocabulary::{CacheState, EventKind};
use support_api::queries::event_kinds;
use support_read::calls::{operation_of, run, wait_finished};
use support_read::mail;
use support_read::world::start_world;

/// Le corps téléchargé est rangé : texte, source brute, état, index de recherche et événement
#[tokio::test]
async fn fetch_message_body_downloads_and_stores_everything() {
    let world = start_world().await;
    let raw = mail::plain("licorne", "Le corps contient un narval.");
    let id = world.deliver_header_only(&raw);
    assert_eq!(world.fetch_body(&id).await, OperationStatus::Finished);
    let engine = &world.fixture.engine;
    let body = Request::Message(MessageRequest::GetMessageBody {
        message: id.clone(),
    });
    let Response::MessageBody(view) = run(engine, body).await.unwrap() else {
        panic!("réponse")
    };
    assert!(view.text.contains("narval"));
    let source = Request::Message(MessageRequest::GetRawSource {
        message: id.clone(),
    });
    let Response::RawSource(stored) = run(engine, source).await.unwrap() else {
        panic!("réponse")
    };
    assert_eq!(STANDARD.decode(stored.content_base64).unwrap(), raw);
    let kinds = event_kinds(engine).await;
    let account = world.account.clone();
    assert!(kinds.contains(&EventKind::BodyAvailable {
        account,
        message: id
    }));
}

/// Un message téléchargé avec une pièce jointe la liste, non encore téléchargée
#[tokio::test]
async fn fetch_message_body_registers_attachments() {
    let world = start_world().await;
    let id = world.deliver_header_only(&mail::with_attachment(1_000));
    assert_eq!(world.fetch_body(&id).await, OperationStatus::Finished);
    let list = Request::Message(MessageRequest::ListAttachments { message: id });
    let Response::Attachments(found) = run(&world.fixture.engine, list).await.unwrap() else {
        panic!("réponse")
    };
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].filename.as_deref(), Some("data.bin"));
    assert_eq!(found[0].size, 1_000);
    assert_eq!(found[0].cache, CacheState::NotDownloaded);
}

/// Une opération annulée avant son premier point de contrôle ne range rien
#[tokio::test]
async fn cancelled_fetch_stores_nothing() {
    let world = start_world().await;
    let id = world.deliver_header_only(&mail::plain("annule", "corps"));
    let engine = &world.fixture.engine;
    let request = Request::Message(MessageRequest::FetchMessageBody {
        message: id.clone(),
    });
    let operation = operation_of(run(engine, request).await.unwrap());
    let cancel = Request::Operation(OperationRequest::CancelOperation {
        operation: operation.clone(),
    });
    run(engine, cancel).await.unwrap();
    assert_eq!(
        wait_finished(engine, &operation).await,
        OperationStatus::Cancelled
    );
    let body = Request::Message(MessageRequest::GetMessageBody { message: id });
    assert!(run(engine, body).await.is_err());
}

/// Télécharger un corps déjà présent ne rouvre aucune connexion
#[tokio::test]
async fn fetch_of_present_body_does_not_connect() {
    let world = start_world().await;
    let id = world.deliver_header_only(&mail::plain("present", "corps"));
    assert_eq!(world.fetch_body(&id).await, OperationStatus::Finished);
    let connections = world.fixture.imap.connection_count();
    assert_eq!(world.fetch_body(&id).await, OperationStatus::Finished);
    assert_eq!(world.fixture.imap.connection_count(), connections);
}

/// Un message absent du serveur donne une opération en échec `not_found.message`
#[tokio::test]
async fn fetch_of_message_gone_from_server_fails() {
    let world = start_world().await;
    let id = world.deliver_header_only(&mail::plain("parti", "corps"));
    world.fixture.store.expunge(world.user(), "INBOX", 1);
    let status = world.fetch_body(&id).await;
    assert_eq!(
        status,
        OperationStatus::Failed {
            code: "not_found.message".into()
        }
    );
}
