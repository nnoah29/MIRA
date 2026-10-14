#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! HTML sûr et autorisation du contenu distant (décision 10) : blocage par défaut, autorisation par message ou expéditeur.

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use mira_core::api::{MessageRequest, RemoteContentTarget, Request, Response};
use mira_vocabulary::MessageId;
use support_read::calls::run;
use support_read::html::{safe_html, seed_remote_pair};
use support_read::seed::add_body;
use support_read::world::start_world;

/// Par défaut, l'image distante est retirée et listée parmi les ressources bloquées
#[tokio::test]
async fn safe_html_blocks_remote_resources_by_default() {
    let world = start_world().await;
    seed_remote_pair(&world);
    let safe = safe_html(&world, "m1").await;
    assert_eq!(
        safe.blocked_resources,
        vec!["https://tracker.example/pixel.gif"]
    );
    assert!(!safe.html.contains("tracker.example"));
    assert!(safe.html.contains("Bonjour"));
}

/// L'autorisation d'un message est mémorisée en base et ne touche pas les autres messages
#[tokio::test]
async fn allow_remote_content_for_a_message_is_persisted() {
    let world = start_world().await;
    seed_remote_pair(&world);
    let scope = RemoteContentTarget::Message {
        message: MessageId::from_raw("m1"),
    };
    let allowed = run(
        &world.fixture.engine,
        Request::Message(MessageRequest::AllowRemoteContent { scope }),
    )
    .await;
    assert_eq!(allowed.unwrap(), Response::Done);
    assert!(safe_html(&world, "m1").await.blocked_resources.is_empty());
    assert!(
        safe_html(&world, "m1")
            .await
            .html
            .contains("tracker.example")
    );
    assert_eq!(safe_html(&world, "m2").await.blocked_resources.len(), 1);
    let stored = world
        .mirror
        .read(|tx| tx.list_remote_content_allowances())
        .unwrap();
    assert_eq!(stored.len(), 1);
}

/// L'autorisation d'un expéditeur (adresse insensible à la casse) couvre tous ses messages
#[tokio::test]
async fn allow_remote_content_for_a_sender_covers_all_its_messages() {
    let world = start_world().await;
    seed_remote_pair(&world);
    let scope = RemoteContentTarget::Sender {
        address: "Bob@Example.org".into(),
    };
    run(
        &world.fixture.engine,
        Request::Message(MessageRequest::AllowRemoteContent { scope }),
    )
    .await
    .unwrap();
    for id in ["m1", "m2"] {
        assert!(
            safe_html(&world, id).await.blocked_resources.is_empty(),
            "{id}"
        );
    }
}

/// Un message inconnu ou une adresse invalide sont refusés
#[tokio::test]
async fn allow_remote_content_refuses_unknown_message_and_bad_address() {
    let world = start_world().await;
    let engine = &world.fixture.engine;
    let unknown = RemoteContentTarget::Message {
        message: MessageId::from_raw("absent"),
    };
    let error = run(
        engine,
        Request::Message(MessageRequest::AllowRemoteContent { scope: unknown }),
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, "not_found.message");
    let bad = RemoteContentTarget::Sender {
        address: "pas une adresse".into(),
    };
    let error = run(
        engine,
        Request::Message(MessageRequest::AllowRemoteContent { scope: bad }),
    )
    .await
    .unwrap_err();
    assert_eq!(error.category, mira_vocabulary::ErrorCategory::InvalidInput);
}

/// Un message sans HTML rend un HTML vide ; sans corps local, l'erreur hors ligne
#[tokio::test]
async fn safe_html_of_text_only_and_headers_only_messages() {
    let world = start_world().await;
    world.seed_messages(2);
    add_body(
        &world.mirror,
        &MessageId::from_raw("m1"),
        "texte seul",
        None,
    );
    let text_only = safe_html(&world, "m1").await;
    assert!(text_only.html.is_empty() && text_only.blocked_resources.is_empty());
    let request = Request::Message(MessageRequest::GetSafeHtml {
        message: MessageId::from_raw("m2"),
    });
    let error = run(&world.fixture.engine, request).await.unwrap_err();
    assert_eq!(error.code, "incompatible_state.body_unavailable_offline");
}
