//! Clavier de la vue « Brouillons » : `B` seulement depuis la liste et le lecteur, vue modale, suppression en deux temps.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod drafts_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use drafts_fixtures::{named, open_drafts_with};
use input_fixtures::{code, ctrl, key, map, type_keys};
use mira_tui::state::{Pane, Popup};
use mira_tui::update::{ComposeIntent, DraftsIntent, EditKey, Intent, SearchIntent};
use navigation_fixtures::press;

/// `B` ouvre la vue depuis la liste et le lecteur ; ailleurs c'est du texte ou rien.
#[test]
fn b_key_opens_drafts_only_from_list_and_reader() {
    let open = Some(Intent::Drafts(DraftsIntent::Open));
    let mut state = state_with_mailboxes();
    assert_eq!(map(&state, key('B')), open);
    state.focus = Pane::Reader;
    assert_eq!(map(&state, key('B')), open);
    state.focus = Pane::Sidebar;
    assert_eq!(map(&state, key('B')), None);
    let editing = compose_fixtures::open_reply();
    let typed = Intent::Compose(ComposeIntent::Edit(EditKey::Char('B')));
    assert_eq!(map(&editing, key('B')), Some(typed));
    let mut creating = state_with_mailboxes();
    press(&mut creating, Intent::Compose(ComposeIntent::Reply));
    assert_eq!(map(&creating, key('B')), None);
    let mut searching = state_with_mailboxes();
    press(&mut searching, Intent::Search(SearchIntent::Open));
    assert_eq!(
        map(&searching, key('B')),
        Some(Intent::Search(SearchIntent::Char('B')))
    );
}

/// Vue ouverte : les touches du courrier n'agissent jamais sur l'arrière-plan.
#[test]
fn drafts_view_is_modal() {
    let mut state = state_with_mailboxes();
    open_drafts_with(&mut state, named(&["A"]));
    let before = state.messages.clone();
    for c in [
        'd', 'e', 's', 'u', 'm', 'z', '/', '?', 'Q', 'c', 'r', 'a', 'f', 'B', ']',
    ] {
        let commands = type_keys(&mut state, &[key(c)]);
        assert!(commands.is_empty(), "{c} : {commands:?}");
        assert_eq!(state.popup, Popup::Closed, "{c}");
        assert!(state.compose.is_closed() && !state.queue.is_open(), "{c}");
        assert!(!state.search.is_editing() && state.drafts.is_open(), "{c}");
    }
    assert!(type_keys(&mut state, &[ctrl('s'), code(KeyCode::Tab)]).is_empty());
    assert_eq!(state.messages, before);
    for intent in [
        Intent::Archive,
        Intent::Trash,
        Intent::ToggleFlag,
        Intent::Undo,
    ] {
        assert!(press(&mut state, intent).is_empty(), "{intent:?}");
    }
    assert!(state.drafts.is_open());
}

/// Touches de la vue : déplacements, Entrée, `D`, Échap, `q`.
#[test]
fn drafts_view_keys() {
    let mut state = state_with_mailboxes();
    open_drafts_with(&mut state, named(&["A"]));
    let wanted = [
        (key('j'), Intent::MoveDown),
        (code(KeyCode::Down), Intent::MoveDown),
        (key('k'), Intent::MoveUp),
        (code(KeyCode::Up), Intent::MoveUp),
        (key('g'), Intent::Top),
        (key('G'), Intent::Bottom),
        (code(KeyCode::Enter), Intent::Open),
        (key('D'), Intent::Drafts(DraftsIntent::AskDelete)),
        (code(KeyCode::Esc), Intent::Back),
        (key('q'), Intent::Quit),
    ];
    for (event, intent) in wanted {
        assert_eq!(map(&state, event), Some(intent), "{event:?}");
    }
    let alt_d = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::ALT | KeyModifiers::SHIFT);
    assert_eq!(map(&state, alt_d), None);
}

/// Confirmation de suppression : seul un `D` sans Ctrl ni Alt supprime, toute autre touche annule.
#[test]
fn confirm_delete_keys() {
    let mut state = state_with_mailboxes();
    open_drafts_with(&mut state, named(&["A"]));
    type_keys(&mut state, &[key('D')]);
    assert!(matches!(state.popup, Popup::ConfirmDeleteDraft { .. }));
    let confirm = Some(Intent::Drafts(DraftsIntent::ConfirmDelete));
    assert_eq!(map(&state, key('D')), confirm);
    for other in [
        key('d'),
        key('x'),
        key('y'),
        code(KeyCode::Enter),
        code(KeyCode::Esc),
        ctrl('D'),
    ] {
        assert_eq!(map(&state, other), Some(Intent::Cancel), "{other:?}");
    }
}
