//! Changer de boîte, de compte ou lancer une recherche vide le lecteur ; le déplacement ne propose pas la boîte du message.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use message_fixtures::summary;
use mira_assembly::mira_vocabulary::MailboxId;
use mira_tui::state::{AppState, Pane, Popup};
use mira_tui::update::{Intent, SearchIntent};
use navigation_fixtures::press;
use search_fixtures::type_text;
use sidebar_fixtures::account;

/// État avec m1 ouvert dans le lecteur.
fn reading() -> AppState {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Open);
    assert!(state.reader.message().is_some());
    state
}

/// Ouvrir une boîte, changer de compte ou chercher vide le lecteur.
#[test]
fn changing_view_clears_reader() {
    let mut state = reading();
    state.focus = Pane::Sidebar;
    press(&mut state, Intent::Open);
    assert!(state.reader.message().is_none(), "boîte");
    let mut state = reading();
    state
        .accounts
        .replace_items(vec![account("a1"), account("a2")]);
    press(&mut state, Intent::NextAccount);
    assert!(state.reader.message().is_none(), "compte");
    let mut state = reading();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "x");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    assert!(state.reader.message().is_none(), "recherche");
}

/// Pendant une recherche, le sélecteur n'offre pas la boîte où le message se trouve déjà.
#[test]
fn move_picker_excludes_message_mailboxes() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "x");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    let mut found = summary("m1", "Bonjour", "Bob");
    found.mailboxes = vec![MailboxId::from_raw("a1-INBOX")];
    state.messages.replace_items(vec![found]);
    press(&mut state, Intent::OpenMovePicker);
    let Popup::MovePicker(picker) = &state.popup else {
        panic!("sélecteur fermé : {:?}", state.popup);
    };
    let names: Vec<&str> = picker.targets().iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["Archive", "Projets"]);
}
