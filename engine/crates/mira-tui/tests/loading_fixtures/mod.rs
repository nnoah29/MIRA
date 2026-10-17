//! Fabriques de commandes et d'états pour les tests de chargement des messages.
#![allow(dead_code)]

use crate::sidebar_fixtures::{mailbox, refresh};
use mira_assembly::api::{MessageRequest, Request, Response};
use mira_assembly::mira_vocabulary::{AccountId, MailboxId, MailboxRole, ViewRef};
use mira_assembly::reading::ListQuery;
use mira_tui::commands::{Command, Tag};
use mira_tui::state::AppState;
use mira_tui::update::{Msg, Reply, update};

/// Commande attendue pour charger la première page d'une boîte.
pub fn load_command(mailbox_id: &str) -> Command {
    let view = ViewRef::Mailbox {
        mailbox: MailboxId::from_raw(mailbox_id),
    };
    Command::Execute {
        tag: Tag::Messages {
            view: view.clone(),
            after: None,
        },
        request: Request::Message(MessageRequest::ListMessages(ListQuery::of(view))),
    }
}

/// Applique la réponse des boîtes du compte a1 (réception puis envoyés).
pub fn mailboxes_arrive(state: &mut AppState) -> Vec<Command> {
    let boxes = vec![
        mailbox("a1", "INBOX", MailboxRole::Inbox, 2),
        mailbox("a1", "Sent", MailboxRole::Sent, 0),
    ];
    let reply = Reply {
        tag: Tag::Mailboxes(AccountId::from_raw("a1")),
        result: Ok(Response::Mailboxes(boxes)),
    };
    update(state, Msg::Reply(reply))
}

/// État avec les comptes a1 et a2 chargés et les boîtes de a1 reçues.
pub fn loaded() -> AppState {
    let mut state = AppState::new(100, 30, 1);
    refresh(&mut state, &["a1", "a2"]);
    mailboxes_arrive(&mut state);
    state
}
