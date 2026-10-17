//! Une requête dont la tâche panique rend quand même une réponse d'erreur, qui débloque l'état.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{IdempotencyKey, MiraError};
use mira_tui::app::request_task::RequestTasks;
use mira_tui::commands::Tag;
use mira_tui::state::AppState;
use mira_tui::update::{Msg, Reply, update};
use tokio::sync::mpsc;

/// Requête simulée qui panique au lieu de répondre.
async fn exploding() -> Result<Response, MiraError> {
    panic!("échec simulé de la requête")
}

/// Lance la requête qui panique sous cette étiquette et rend la réponse produite pour elle.
async fn reply_of_panicking_request(tag: Tag) -> Reply {
    let (sender, mut receiver) = mpsc::unbounded_channel();
    let mut tasks = RequestTasks::new(sender);
    tasks.spawn(tag, exploding());
    assert!(tasks.settle_next().await.is_some());
    match receiver.try_recv().unwrap() {
        Msg::Reply(reply) => reply,
        other => panic!("message inattendu : {other:?}"),
    }
}

/// La panique devient une erreur interne étiquetée, qui libère le marqueur « état en cours de lecture ».
#[tokio::test]
async fn panicking_request_replies_with_internal_error() {
    let reply = reply_of_panicking_request(Tag::Status).await;
    assert_eq!(reply.tag, Tag::Status);
    let error = reply.result.as_ref().unwrap_err();
    assert_eq!(error.code, "internal.bug");
    assert!(!error.message.contains("échec simulé"), "{}", error.message);
    let mut state = AppState::new(100, 30, 1);
    state.refresh.status = true;
    update(&mut state, Msg::Reply(reply));
    assert!(!state.refresh.status);
    assert!(state.status.message.is_some());
}

/// Une action dont la tâche panique n'est plus « en vol » : `z` redevient possible.
#[tokio::test]
async fn panicking_action_no_longer_blocks_undo() {
    let key = IdempotencyKey("tui-1-act-1".to_string());
    let reply = reply_of_panicking_request(Tag::UserAction(key.clone())).await;
    let mut state = AppState::new(100, 30, 1);
    state.actions.await_reply(key);
    update(&mut state, Msg::Reply(reply));
    assert!(!state.actions.in_flight());
}

/// Une requête qui réussit répond une seule fois, sans réponse d'erreur en plus.
#[tokio::test]
async fn successful_request_replies_once() {
    let (sender, mut receiver) = mpsc::unbounded_channel();
    let mut tasks = RequestTasks::new(sender);
    tasks.spawn(Tag::Status, async { Ok(Response::Done) });
    assert!(tasks.settle_next().await.is_some());
    assert!(matches!(
        receiver.try_recv(),
        Ok(Msg::Reply(Reply { result: Ok(_), .. }))
    ));
    assert!(receiver.try_recv().is_err());
    assert!(tasks.settle_next().await.is_none());
}

/// L'arrêt détaché rend la main tout de suite ; la tâche bloquée est interrompue après le délai accordé.
#[tokio::test]
async fn detached_stop_returns_at_once_and_finishes_after_deadline() {
    let (sender, _receiver) = mpsc::unbounded_channel();
    let mut tasks = RequestTasks::new(sender);
    tasks.spawn(Tag::Status, async {
        tokio::time::sleep(std::time::Duration::from_secs(3_600)).await;
        Ok(Response::Done)
    });
    let started = tokio::time::Instant::now();
    let stopping = tasks.stop_detached();
    assert!(started.elapsed() < std::time::Duration::from_millis(100));
    stopping.await.unwrap();
    assert!(started.elapsed() < std::time::Duration::from_secs(10));
}
