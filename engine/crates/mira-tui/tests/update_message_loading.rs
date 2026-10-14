//! Chargement des messages d'une boîte : ouverture, chargement automatique, vue périmée.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod loading_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod paging_fixtures;
mod sidebar_fixtures;

use loading_fixtures::{load_command, loaded, mailboxes_arrive};
use message_fixtures::{inbox_view, page};
use mira_assembly::api::Response;
use mira_tui::commands::Tag;
use mira_tui::state::{AppState, Pane};
use mira_tui::update::{Intent, Msg, Reply, update};
use navigation_fixtures::press;
use paging_fixtures::messages_reply;
use sidebar_fixtures::refresh;

/// Ouvrir une boîte depuis la barre latérale charge sa première page sans curseur.
#[test]
fn opening_a_mailbox_emits_list_messages_without_cursor() {
    let mut state = loaded();
    state.messages.replace_page(inbox_view(), page(0, 3, None));
    state.mailboxes.selection.selected = Some(1);
    let commands = press(&mut state, Intent::Open);
    assert_eq!(commands, vec![load_command("a1-Sent")]);
    assert_eq!(state.focus, Pane::List);
    assert!(state.messages.is_empty());
}

/// Les boîtes reçues déclenchent le chargement de la première, une seule fois.
#[test]
fn mailboxes_arrival_loads_first_mailbox_once() {
    let mut state = AppState::new(100, 30);
    refresh(&mut state, &["a1"]);
    assert_eq!(mailboxes_arrive(&mut state), vec![load_command("a1-INBOX")]);
    assert!(mailboxes_arrive(&mut state).is_empty());
}

/// Changer de compte vide la liste et oublie la vue voulue.
#[test]
fn account_switch_clears_message_list() {
    let mut state = loaded();
    update(
        &mut state,
        Msg::Reply(Reply {
            tag: Tag::Messages {
                view: inbox_view(),
                after: None,
            },
            result: Ok(Response::Messages(page(0, 3, None))),
        }),
    );
    assert_eq!(state.messages.len(), 3);
    press(&mut state, Intent::NextAccount);
    assert!(state.messages.is_empty());
    assert!(state.messages.view().is_none());
}

/// Une réponse tardive d'une boîte abandonnée laisse la liste de la boîte courante intacte.
#[test]
fn late_reply_of_previous_mailbox_is_ignored() {
    let mut state = loaded();
    state.mailboxes.selection.selected = Some(1);
    press(&mut state, Intent::Open);
    update(&mut state, messages_reply(None, Ok(page(0, 5, None))));
    assert!(state.messages.is_empty());
}
