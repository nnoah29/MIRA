//! Fabriques de commandes, de réponses et de saisie pour les tests de recherche.
#![allow(dead_code)]

use crate::message_fixtures::numbered;
use mira_assembly::api::{Request, Response, SearchRequest};
use mira_assembly::mira_vocabulary::PageRequest;
use mira_assembly::mira_vocabulary::{Cursor, Page};
use mira_assembly::reading::SearchFilters;
use mira_tui::commands::{Command, Tag};
use mira_tui::state::AppState;
use mira_tui::update::{Intent, Msg, Reply, SearchIntent, update};

/// Tape ce texte caractère par caractère dans la saisie de recherche.
pub fn type_text(state: &mut AppState, text: &str) {
    for c in text.chars() {
        update(state, Msg::Intent(Intent::Search(SearchIntent::Char(c))));
    }
}

/// Commande attendue pour chercher cette requête avec les filtres et la page par défaut.
pub fn search_command(query: &str) -> Command {
    Command::Execute {
        tag: Tag::Search(query.to_string()),
        request: Request::Search(SearchRequest::Search {
            query: query.to_string(),
            filters: SearchFilters::default(),
            page: PageRequest::default(),
        }),
    }
}

/// Réponse du moteur à la recherche de cette requête, avec `count` résultats.
pub fn search_reply(query: &str, count: usize) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::Search(query.to_string()),
        result: Ok(Response::Messages(Page {
            items: numbered(count),
            next_cursor: None::<Cursor>,
        })),
    })
}
