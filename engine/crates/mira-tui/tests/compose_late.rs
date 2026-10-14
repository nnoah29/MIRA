//! Brouillon : réponses tardives ignorées, rechargements automatiques sans effet sur l'édition.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod engine_fixtures;
mod loading_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::{action_reply, state_with_mailboxes};
use compose_fixtures::{compose, created_reply, draft, focus_on, open_reply, type_in_draft};
use engine_fixtures::sync_finished;
use loading_fixtures::mailboxes_arrive;
use mira_tui::state::AppState;
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, Intent, Msg, SearchIntent, update};
use navigation_fixtures::press;
use reader_fixtures::offline_error;
use sidebar_fixtures::refresh;

/// Éditeur ouvert avec un objet et un corps modifiés.
fn typing() -> AppState {
    let mut state = open_reply();
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "!");
    focus_on(&mut state, Field::Body);
    type_in_draft(&mut state, "ligne\nsuite");
    state
}

/// La réponse de création d'une autre demande ne remplace ni n'ouvre rien.
#[test]
fn created_reply_for_another_request_is_ignored() {
    let mut state = state_with_mailboxes();
    compose(&mut state, ComposeIntent::Reply);
    update(
        &mut state,
        created_reply("tui-1-draft-99", draft("d2", &[], "x", "")),
    );
    assert!(state.compose.is_creating() && state.compose.open().is_none());
    let mut open = typing();
    let before = open.clone();
    update(
        &mut open,
        created_reply("tui-1-draft-1", draft("d2", &[], "x", "")),
    );
    assert_eq!(open, before);
}

/// Une création refusée referme l'attente, montre l'erreur, et permet d'en demander une autre.
#[test]
fn failed_creation_reports_and_allows_a_new_request() {
    let mut state = state_with_mailboxes();
    compose(&mut state, ComposeIntent::Reply);
    let reply = mira_tui::update::Reply {
        tag: mira_tui::commands::Tag::DraftCreated(mira_assembly::mira_vocabulary::IdempotencyKey(
            "tui-1-draft-1".to_string(),
        )),
        result: Err(offline_error()),
    };
    update(&mut state, Msg::Reply(reply));
    assert!(state.compose.is_closed());
    assert!(state.status.message.is_some());
    assert_eq!(compose(&mut state, ComposeIntent::New).len(), 1);
}

/// Événement du moteur, boîtes rechargées et échec d'action ne ferment ni ne vident le brouillon.
#[test]
fn automatic_reloads_do_not_close_or_clear_the_draft() {
    let mut state = typing();
    let before = state.compose.clone();
    update(&mut state, sync_finished("a1"));
    mailboxes_arrive(&mut state);
    refresh(&mut state, &["a1", "a2"]);
    update(
        &mut state,
        action_reply("tui-1-act-1", Err(offline_error())),
    );
    update(&mut state, Msg::Resize(60, 20));
    assert_eq!(state.compose, before);
    assert!(state.compose.open().unwrap().has_unsaved());
}

/// Pendant l'édition, aucune action de la liste ne part et la recherche ne s'ouvre pas.
#[test]
fn other_intents_do_nothing_while_editing() {
    let mut state = typing();
    let before = state.clone();
    for intent in [
        Intent::Archive,
        Intent::Trash,
        Intent::DeletePermanently,
        Intent::OpenMovePicker,
        Intent::Search(SearchIntent::Open),
        Intent::Help,
        Intent::NextPane,
        Intent::Open,
    ] {
        assert!(press(&mut state, intent).is_empty(), "{intent:?}");
    }
    assert_eq!(state, before);
}

/// Une saisie de recherche ouverte pendant la création est fermée quand l'éditeur s'ouvre.
#[test]
fn opening_the_editor_closes_a_search_input_and_popups() {
    let mut state = state_with_mailboxes();
    compose(&mut state, ComposeIntent::Reply);
    press(&mut state, Intent::Search(SearchIntent::Open));
    assert!(state.search.is_editing());
    update(
        &mut state,
        created_reply("tui-1-draft-1", draft("d1", &[], "x", "")),
    );
    assert!(state.compose.open().is_some());
    assert!(!state.search.is_editing());
}
