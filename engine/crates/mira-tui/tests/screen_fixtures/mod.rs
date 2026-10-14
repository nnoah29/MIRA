//! Tous les écrans et fenêtres du TUI, pour les tests de rendu (petits terminaux, aide, mode étroit).
#![allow(dead_code)]

use crate::action_fixtures::state_with_mailboxes;
use crate::navigation_fixtures::press;
use crate::reader_fixtures::{body, body_reply, offline_error};
use crate::search_fixtures::{search_reply, type_text};
use crate::sidebar_fixtures::loaded_state;
use mira_tui::state::{AppState, Pane};
use mira_tui::update::{Intent, SearchIntent, update};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// Écrans principaux : accueil, barre latérale, liste, lecteur avec corps ou hors ligne.
fn main_screens() -> Vec<(&'static str, AppState)> {
    let mut reader = state_with_mailboxes();
    press(&mut reader, Intent::Open);
    update(&mut reader, body_reply("m1", Ok(body("Ligne 1\nLigne 2"))));
    let mut offline = state_with_mailboxes();
    press(&mut offline, Intent::Open);
    update(&mut offline, body_reply("m1", Err(offline_error())));
    let mut busy = state_with_mailboxes();
    busy.status.message = Some("erreur du moteur".to_string());
    busy.status.syncing = true;
    busy.status.offline = true;
    busy.status.pending_actions = 3;
    vec![
        ("accueil", AppState::new(100, 30, 1)),
        ("barre latérale", loaded_state()),
        ("liste", state_with_mailboxes()),
        ("lecteur", reader),
        ("lecteur hors ligne", offline),
        ("barre d'état chargée", busy),
    ]
}

/// Écrans de recherche : saisie ouverte et résultats affichés.
fn search_screens() -> Vec<(&'static str, AppState)> {
    let mut typing = state_with_mailboxes();
    press(&mut typing, Intent::Search(SearchIntent::Open));
    type_text(&mut typing, "facture");
    let mut results = typing.clone();
    press(&mut results, Intent::Search(SearchIntent::Submit));
    update(&mut results, search_reply("facture", 3));
    vec![("saisie de recherche", typing), ("résultats", results)]
}

/// Fenêtres modales : confirmation, déplacement et aide de chaque panneau.
fn popup_screens() -> Vec<(&'static str, AppState)> {
    let mut confirm = state_with_mailboxes();
    press(&mut confirm, Intent::DeletePermanently);
    let mut picker = state_with_mailboxes();
    press(&mut picker, Intent::OpenMovePicker);
    let mut screens = vec![("confirmation", confirm), ("déplacement", picker)];
    for (name, pane) in [
        ("aide barre latérale", Pane::Sidebar),
        ("aide liste", Pane::List),
        ("aide lecteur", Pane::Reader),
    ] {
        let mut help = state_with_mailboxes();
        help.focus = pane;
        press(&mut help, Intent::Help);
        screens.push((name, help));
    }
    screens
}

/// Tous les écrans connus, nommés pour les messages d'échec.
pub fn all_screens() -> Vec<(&'static str, AppState)> {
    let mut screens = main_screens();
    screens.extend(search_screens());
    screens.extend(popup_screens());
    screens
}

/// Rend l'état dans un terminal de cette taille et rend le texte affiché, ligne par ligne.
pub fn text_in(state: &AppState, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| mira_tui::view::render(frame, state))
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    let symbols: Vec<&str> = buffer.content().iter().map(|cell| cell.symbol()).collect();
    symbols
        .chunks(usize::from(width.max(1)))
        .map(|row| row.concat())
        .collect::<Vec<_>>()
        .join("\n")
}
