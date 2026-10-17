//! Démarrage d'un brouillon : répondre, répondre à tous, transférer, nouveau message.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;
mod target_fixtures;

use action_fixtures::state_with_mailboxes;
use compose_fixtures::{compose, create_command};
use mira_assembly::api::DraftMode;
use mira_tui::state::AppState;
use mira_tui::update::ComposeIntent;
use target_fixtures::{id, reader_on_m1_list_on_m2};

/// Répondre crée un brouillon en mode réponse pour le message sélectionné de la liste.
#[test]
fn reply_creates_draft_with_reply_mode_for_selected_message() {
    let mut state = state_with_mailboxes();
    let commands = compose(&mut state, ComposeIntent::Reply);
    let mode = DraftMode::Reply { message: id("m1") };
    assert_eq!(commands, vec![create_command("a1", mode, "tui-1-draft-1")]);
    assert!(state.compose.is_creating());
    assert!(state.compose.open().is_none());
}

/// Répondre à tous et transférer demandent leur mode au moteur.
#[test]
fn reply_all_and_forward_create_their_own_modes() {
    let mut state = state_with_mailboxes();
    let all = DraftMode::ReplyAll { message: id("m1") };
    let commands = compose(&mut state, ComposeIntent::ReplyAll);
    assert_eq!(commands, vec![create_command("a1", all, "tui-1-draft-1")]);
    let mut state = state_with_mailboxes();
    let forward = DraftMode::Forward { message: id("m1") };
    let commands = compose(&mut state, ComposeIntent::Forward);
    assert_eq!(
        commands,
        vec![create_command("a1", forward, "tui-1-draft-1")]
    );
}

/// Depuis le lecteur, la réponse vise le message ouvert et non la ligne sélectionnée de la liste.
#[test]
fn reply_from_reader_targets_open_message() {
    let mut state = reader_on_m1_list_on_m2();
    assert_eq!(state.messages.selection.selected, Some(1));
    let commands = compose(&mut state, ComposeIntent::Reply);
    let mode = DraftMode::Reply { message: id("m1") };
    assert_eq!(commands, vec![create_command("a1", mode, "tui-1-draft-2")]);
}

/// Un nouveau message se crée dans le compte sélectionné, sans message d'origine.
#[test]
fn new_message_uses_selected_account() {
    let mut state = state_with_mailboxes();
    let commands = compose(&mut state, ComposeIntent::New);
    let expected = create_command("a1", DraftMode::New, "tui-1-draft-1");
    assert_eq!(commands, vec![expected]);
}

/// Une deuxième demande pendant la création n'en lance pas une autre.
#[test]
fn second_request_while_creating_is_ignored() {
    let mut state = state_with_mailboxes();
    compose(&mut state, ComposeIntent::Reply);
    assert!(compose(&mut state, ComposeIntent::Reply).is_empty());
    assert!(compose(&mut state, ComposeIntent::New).is_empty());
}

/// Sans message ni compte, rien n'est demandé et la barre d'état le dit.
#[test]
fn nothing_to_reply_to_or_no_account_sends_nothing() {
    let mut state = AppState::new(80, 24, 1);
    assert!(compose(&mut state, ComposeIntent::Reply).is_empty());
    assert!(state.status.message.is_some());
    state.status.message = None;
    assert!(compose(&mut state, ComposeIntent::New).is_empty());
    assert!(state.status.message.is_some());
    assert!(state.compose.is_closed());
}
