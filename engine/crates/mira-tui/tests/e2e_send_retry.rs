//! Envoi refusé puis renvoyé : le TUI réutilise la même clé, le moteur dédoublonne, le faux SMTP reçoit un seul message.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod e2e_support;
mod send_fixtures;
mod support;

use mira_assembly::api::{
    ComposeRequest, DraftChanges, DraftMode, Engine, Request, Response, WriteResponse,
};
use mira_assembly::mira_vocabulary::error::codes::internal::BUG;
use mira_assembly::mira_vocabulary::{
    AccountId, ActionKind, AddressEntry, Draft, ErrorCategory, MiraError,
};
use mira_tui::commands::Command;
use mira_tui::state::compose::ComposeState;
use mira_tui::state::{AppState, ComposeSlot};
use mira_tui::update::{Msg, Reply, update};
use send_fixtures::send_confirmed;
use support::send_world::SendWorld;

/// Brouillon du compte prêt à partir : bob en destinataire, un objet, un corps.
async fn ready_draft(engine: &Engine, account: AccountId) -> Draft {
    let create = ComposeRequest::CreateDraft {
        account,
        identity: None,
        mode: DraftMode::New,
    };
    let Response::Write(WriteResponse::Draft(draft)) =
        engine.execute(Request::Compose(create)).await.unwrap()
    else {
        panic!("brouillon attendu");
    };
    let changes = DraftChanges {
        to: Some(vec![AddressEntry {
            name: None,
            address: "bob@example.org".to_string(),
        }]),
        subject: Some("Relance".to_string()),
        body_text: Some("Corps".to_string()),
        ..DraftChanges::default()
    };
    let changing = ComposeRequest::UpdateDraft {
        draft: draft.id.clone(),
        changes,
    };
    match engine.execute(Request::Compose(changing)).await.unwrap() {
        Response::Write(WriteResponse::Draft(saved)) => *saved,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Exécute la seule commande produite contre le moteur ; rend sa réponse étiquetée.
async fn run_one(engine: &Engine, commands: Vec<Command>) -> Reply {
    let [Command::Execute { tag, request }] = <[Command; 1]>::try_from(commands).unwrap() else {
        panic!("une requête attendue");
    };
    Reply {
        tag,
        result: engine.execute(request).await,
    }
}

/// État dont l'éditeur est ouvert sur ce brouillon du moteur.
fn editor_on(draft: &Draft) -> AppState {
    let mut state = AppState::new(100, 30, 7);
    state.compose = ComposeSlot::Open(Box::new(ComposeState::from_draft(draft)));
    state
}

/// Le moteur a pris l'envoi mais sa réponse s'est perdue (erreur) : le renvoi part avec la même clé, un seul message.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_send_then_resend_delivers_once() {
    let world = SendWorld::start().await;
    let engine = world.world.engine();
    let account = world.world.account().unwrap();
    let draft = ready_draft(&engine, account.clone()).await;
    let mut state = editor_on(&draft);
    let first = send_confirmed(&mut state);
    let lost = run_one(&engine, first.clone()).await;
    assert!(lost.result.is_ok());
    let failure = MiraError::new(ErrorCategory::Internal, BUG, "réponse perdue");
    let failed = Reply {
        tag: lost.tag,
        result: Err(failure),
    };
    update(&mut state, Msg::Reply(failed));
    let second = send_confirmed(&mut state);
    assert_eq!(second, first);
    let reply = run_one(&engine, second).await;
    update(&mut state, Msg::Reply(reply));
    assert!(state.compose.is_closed());
    let actions = e2e_support::queued_actions(&world.world, account).await;
    let sends = actions
        .iter()
        .filter(|a| matches!(a.kind, ActionKind::Send { .. }))
        .count();
    assert_eq!(sends, 1, "{actions:?}");
    assert_eq!(world.deliver().await, 1);
}
