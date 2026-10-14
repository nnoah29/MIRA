#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Fils, recherche locale avec filtres et recherche étendue au serveur (opération longue annulable).

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use mira_core::api::{OperationRequest, Request, Response, SearchRequest, ThreadRequest};
use mira_core::operations::OperationStatus;
use mira_core::reading::SearchFilters;
use mira_vocabulary::{MessageId, OperationId, PageRequest, ThreadId};
use support_read::calls::{operation_of, run, wait_finished};
use support_read::seed::{add_body, link_threads};
use support_read::server_mail::deliver_unicorn_mails;
use support_read::world::{World, start_world};

/// Lance SearchServer sur le compte du monde et rend l'identifiant de l'opération
async fn start_server_search(world: &World, query: &str) -> OperationId {
    let request = Request::Search(SearchRequest::SearchServer {
        account: world.account.clone(),
        query: query.to_string(),
    });
    operation_of(run(&world.fixture.engine, request).await.unwrap())
}

/// Les fils d'une boîte sont listés, puis un fil est lu avec ses messages
#[tokio::test]
async fn list_and_get_thread_through_the_facade() {
    let world = start_world().await;
    world.seed_messages(3);
    link_threads(&world.mirror, &world.account);
    let engine = &world.fixture.engine;
    let list = Request::Thread(ThreadRequest::ListThreads {
        mailbox: world.mailbox.clone(),
        page: PageRequest::default(),
    });
    let Response::Threads(page) = run(engine, list).await.unwrap() else {
        panic!("réponse")
    };
    assert_eq!(page.items.len(), 2);
    let get = Request::Thread(ThreadRequest::GetThread {
        thread: ThreadId::from_raw("t-r1"),
    });
    let Response::Thread(view) = run(engine, get).await.unwrap() else {
        panic!("réponse")
    };
    assert_eq!(view.messages.len(), 2);
}

/// Un fil inconnu donne `not_found.thread`
#[tokio::test]
async fn get_unknown_thread_is_not_found() {
    let world = start_world().await;
    let unknown = Request::Thread(ThreadRequest::GetThread {
        thread: ThreadId::from_raw("absent"),
    });
    let error = run(&world.fixture.engine, unknown).await.unwrap_err();
    assert_eq!(error.code, "not_found.thread");
}

/// La recherche locale combine le texte et le filtre d'expéditeur
#[tokio::test]
async fn local_search_applies_text_and_sender_filters() {
    let world = start_world().await;
    world.seed_messages(2);
    for id in ["m1", "m2"] {
        let id = MessageId::from_raw(id);
        add_body(&world.mirror, &id, "rapport trimestriel", None);
    }
    let mut filters = SearchFilters::default();
    let search = |filters: SearchFilters| {
        Request::Search(SearchRequest::Search {
            query: "trimestriel".into(),
            filters,
            page: PageRequest::default(),
        })
    };
    let engine = &world.fixture.engine;
    let Response::Messages(all) = run(engine, search(filters.clone())).await.unwrap() else {
        panic!("réponse")
    };
    filters.common.from_contains = Some("zoe".to_string());
    let Response::Messages(none) = run(engine, search(filters)).await.unwrap() else {
        panic!("réponse")
    };
    assert_eq!((all.items.len(), none.items.len()), (2, 0));
}

/// La recherche serveur trouve aussi les messages absents du miroir et relie ceux qui y sont
#[tokio::test]
async fn search_server_finds_messages_beyond_the_local_mirror() {
    let world = start_world().await;
    deliver_unicorn_mails(&world);
    let operation = start_server_search(&world, "licorne").await;
    let engine = &world.fixture.engine;
    assert_eq!(
        wait_finished(engine, &operation).await,
        OperationStatus::Finished
    );
    let result = Request::Search(SearchRequest::GetServerSearchResult { operation });
    let Response::ServerSearch(found) = run(engine, result).await.unwrap() else {
        panic!("réponse")
    };
    let uids: Vec<u32> = found.hits.iter().map(|hit| hit.uid).collect();
    assert_eq!(uids, vec![3, 1]);
    assert_eq!(found.hits[1].message, Some(MessageId::from_raw("m1")));
    assert_eq!(found.hits[0].message, None);
    assert!(!found.truncated);
}

/// Annulée, la recherche serveur n'a pas de résultat à relire
#[tokio::test]
async fn cancelled_server_search_has_no_result() {
    let world = start_world().await;
    let operation = start_server_search(&world, "licorne").await;
    let cancel = Request::Operation(OperationRequest::CancelOperation {
        operation: operation.clone(),
    });
    let engine = &world.fixture.engine;
    run(engine, cancel).await.unwrap();
    assert_eq!(
        wait_finished(engine, &operation).await,
        OperationStatus::Cancelled
    );
    let result = Request::Search(SearchRequest::GetServerSearchResult { operation });
    let error = run(engine, result).await.unwrap_err();
    assert_eq!(error.code, "not_found.operation");
}
