//! Clavier : chaque touche donne l'intention du contexte courant (spec §4.3).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use crossterm::event::KeyCode;
use input_fixtures::{code, ctrl, key, map};
use mira_tui::state::{AppState, Pane};
use mira_tui::update::{ComposeIntent, Intent, SearchIntent, update};
use navigation_fixtures::press;
use search_fixtures::{search_reply, type_text};

/// État sur la liste, focus déplacé sur ce panneau.
fn focused(pane: Pane) -> AppState {
    let mut state = state_with_mailboxes();
    state.focus = pane;
    state
}

/// `j` et ↓ descendent, `k` et ↑ montent, dans chaque panneau.
#[test]
fn j_and_down_map_to_move_down() {
    for pane in [Pane::Sidebar, Pane::List, Pane::Reader] {
        let state = focused(pane);
        assert_eq!(map(&state, key('j')), Some(Intent::MoveDown), "{pane:?}");
        assert_eq!(map(&state, code(KeyCode::Down)), Some(Intent::MoveDown));
        assert_eq!(map(&state, key('k')), Some(Intent::MoveUp));
        assert_eq!(map(&state, code(KeyCode::Up)), Some(Intent::MoveUp));
        assert_eq!(map(&state, key('g')), Some(Intent::Top));
        assert_eq!(map(&state, key('G')), Some(Intent::Bottom));
        assert_eq!(map(&state, code(KeyCode::Enter)), Some(Intent::Open));
    }
}

/// Hors saisie, `r` répond (étape 2) ; en saisie de recherche, `r`, `q`, `j` et `/` sont du texte.
#[test]
fn r_in_reader_maps_to_reply_but_in_search_is_text() {
    let mut state = focused(Pane::Reader);
    assert_eq!(
        map(&state, key('r')),
        Some(Intent::Compose(ComposeIntent::Reply))
    );
    press(&mut state, Intent::Search(SearchIntent::Open));
    for c in ['r', 'q', 'j', '/', '?', 'e', 'D'] {
        let typed = Some(Intent::Search(SearchIntent::Char(c)));
        assert_eq!(map(&state, key(c)), typed, "{c}");
    }
    let backspace = map(&state, code(KeyCode::Backspace));
    assert_eq!(backspace, Some(Intent::Search(SearchIntent::Backspace)));
    let enter = map(&state, code(KeyCode::Enter));
    assert_eq!(enter, Some(Intent::Search(SearchIntent::Submit)));
    let escape = map(&state, code(KeyCode::Esc));
    assert_eq!(escape, Some(Intent::Search(SearchIntent::Leave)));
}

/// Dans la liste comme dans le lecteur, les touches de message donnent leurs actions.
#[test]
fn message_keys_map_to_actions_in_list_and_reader() {
    let table = [
        ('e', Intent::Archive),
        ('d', Intent::Trash),
        ('D', Intent::DeletePermanently),
        ('s', Intent::ToggleFlag),
        ('u', Intent::ToggleRead),
        ('m', Intent::OpenMovePicker),
        ('z', Intent::Undo),
    ];
    for pane in [Pane::List, Pane::Reader] {
        let state = focused(pane);
        for (c, intent) in table {
            assert_eq!(map(&state, key(c)), Some(intent), "{pane:?} {c}");
        }
    }
    assert_eq!(map(&focused(Pane::Sidebar), key('e')), None);
}

/// Touches globales et changement de compte dans la barre latérale.
#[test]
fn global_and_sidebar_keys_map_to_their_intents() {
    let sidebar = focused(Pane::Sidebar);
    assert_eq!(map(&sidebar, key(']')), Some(Intent::NextAccount));
    assert_eq!(map(&sidebar, key('[')), Some(Intent::PrevAccount));
    for pane in [Pane::Sidebar, Pane::List, Pane::Reader] {
        let state = focused(pane);
        assert_eq!(map(&state, code(KeyCode::Tab)), Some(Intent::NextPane));
        assert_eq!(map(&state, code(KeyCode::BackTab)), Some(Intent::PrevPane));
        assert_eq!(map(&state, key('?')), Some(Intent::Help));
        assert_eq!(map(&state, key('q')), Some(Intent::Quit));
        assert_eq!(map(&state, ctrl('r')), Some(Intent::StartSync));
        let open = Some(Intent::Search(SearchIntent::Open));
        assert_eq!(map(&state, key('/')), open);
        assert_eq!(map(&state, code(KeyCode::Esc)), Some(Intent::Back));
    }
}

/// Résultats affichés et saisie fermée : Échap quitte la recherche, puis redevient « retour ».
#[test]
fn escape_leaves_search_results_then_goes_back() {
    let mut state = focused(Pane::List);
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "facture");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    update(&mut state, search_reply("facture", 2));
    let leave = Some(Intent::Search(SearchIntent::Leave));
    assert_eq!(map(&state, code(KeyCode::Esc)), leave);
    let intent = map(&state, code(KeyCode::Esc)).unwrap();
    press(&mut state, intent);
    assert!(!state.search.is_active());
    assert_eq!(map(&state, code(KeyCode::Esc)), Some(Intent::Back));
}
