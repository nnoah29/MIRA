#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lectures hors ligne par la façade (invariant 27, S6) : le faux serveur est arrêté, les lectures répondent.

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use mira_core::api::{MessageRequest, Request, Response, SearchRequest};
use mira_core::operations::OperationStatus;
use mira_vocabulary::MessageId;
use support_read::calls::{messages_page, operation_of, page_of, run, wait_finished};
use support_read::requests::{mailbox_reads, message_reads};
use support_read::seed::add_body;
use support_read::world::{World, start_world};

/// Sans serveur, la recherche serveur échoue avec un code réseau
#[tokio::test]
async fn server_search_fails_with_network_code_when_offline() {
    let world = start_world().await;
    world.fixture.imap.shutdown();
    let engine = &world.fixture.engine;
    let search = Request::Search(SearchRequest::SearchServer {
        account: world.account.clone(),
        query: "licorne".into(),
    });
    let operation = operation_of(run(engine, search).await.unwrap());
    match wait_finished(engine, &operation).await {
        OperationStatus::Failed { code } => assert!(code.starts_with("network."), "{code}"),
        other => panic!("état inattendu : {other:?}"),
    }
}

/// Garnit deux messages : m1 avec son corps HTML, m2 en-têtes seuls
fn seed_two(world: &World) {
    world.seed_messages(2);
    let html = "<p>Bonjour <b>monde</b></p>";
    add_body(
        &world.mirror,
        &MessageId::from_raw("m1"),
        "Bonjour monde licorne",
        Some(html),
    );
}

/// Invariant 27 : listes, lecture, recherche et fils répondent sans aucune connexion au serveur
#[tokio::test]
async fn invariant_27_list_and_read_work_offline() {
    let world = start_world().await;
    seed_two(&world);
    let before = world.fixture.imap.connection_count();
    world.fixture.imap.shutdown();
    let engine = &world.fixture.engine;
    let mut reads = message_reads(&MessageId::from_raw("m1"));
    reads.extend(mailbox_reads(&world, "licorne"));
    let mut answers = Vec::new();
    for request in reads {
        answers.push(run(engine, request).await.unwrap());
    }
    let page = messages_page(engine, &world.mailbox, 10, None).await;
    assert_eq!(page_of(page.unwrap()).items.len(), 2);
    let found = answers.into_iter().find_map(|answer| match answer {
        Response::Messages(page) => Some(page),
        _ => None,
    });
    assert_eq!(found.unwrap().items.len(), 1);
    assert_eq!(world.fixture.imap.connection_count(), before);
}

/// S6 : corps présent lu tout de suite ; en-têtes seuls signalés ; le téléchargement hors ligne échoue proprement
#[tokio::test]
async fn scenario_s6_read_body_offline_through_the_facade() {
    let world = start_world().await;
    seed_two(&world);
    world.fixture.imap.shutdown();
    let engine = &world.fixture.engine;
    let body = |id: &str| {
        Request::Message(MessageRequest::GetMessageBody {
            message: MessageId::from_raw(id),
        })
    };
    let Response::MessageBody(view) = run(engine, body("m1")).await.unwrap() else {
        panic!("réponse")
    };
    assert_eq!(view.text, "Bonjour monde licorne");
    assert!(view.html_available);
    let missing = run(engine, body("m2")).await.unwrap_err();
    assert_eq!(missing.code, "incompatible_state.body_unavailable_offline");
    let fetch = Request::Message(MessageRequest::FetchMessageBody {
        message: MessageId::from_raw("m2"),
    });
    let operation = operation_of(run(engine, fetch).await.unwrap());
    match wait_finished(engine, &operation).await {
        OperationStatus::Failed { code } => assert!(code.starts_with("network."), "{code}"),
        other => panic!("état inattendu : {other:?}"),
    }
    assert_eq!(
        run(engine, body("m2")).await.unwrap_err().code,
        missing.code
    );
}
