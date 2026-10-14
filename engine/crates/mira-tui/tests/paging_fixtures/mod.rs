//! Fabriques de listes paginées et de requêtes attendues pour les tests de défilement.
#![allow(dead_code)]

use crate::message_fixtures::{inbox_view, page};
use mira_assembly::api::{MessageRequest, Request, Response};
use mira_assembly::mira_vocabulary::{Cursor, MessageSummary, MiraError, Page};
use mira_assembly::reading::ListQuery;
use mira_tui::commands::{Command, Tag};
use mira_tui::state::{AppState, Pane};
use mira_tui::update::{Msg, Reply};

/// État sur la liste avec une première page de 20 messages et un curseur suivant.
pub fn paged_state() -> AppState {
    let mut state = AppState::new(100, 30);
    state
        .messages
        .replace_page(inbox_view(), page(0, 20, Some("c1")));
    state.focus = Pane::List;
    state
}

/// Commande attendue pour charger la page suivante après ce curseur.
pub fn next_page_command(cursor: &str) -> Command {
    let mut query = ListQuery::of(inbox_view());
    query.page.cursor = Some(Cursor(cursor.to_string()));
    Command::Execute {
        tag: Tag::Messages {
            view: inbox_view(),
            after: Some(Cursor(cursor.to_string())),
        },
        request: Request::Message(MessageRequest::ListMessages(query)),
    }
}

/// Réponse d'une page de messages pour la vue de réception.
pub fn messages_reply(after: Option<&str>, page: Result<Page<MessageSummary>, MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::Messages {
            view: inbox_view(),
            after: after.map(|value| Cursor(value.to_string())),
        },
        result: page.map(Response::Messages),
    })
}
