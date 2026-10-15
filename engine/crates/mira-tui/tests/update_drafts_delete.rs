//! Vue « Brouillons » : suppression par `D` puis `D` immédiat, toute autre touche annule, refus du moteur affiché.
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
use compose_fixtures::{delete_command, deleted_reply};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use drafts_fixtures::{drafts_reply, list_command, named, not_editable_error, open_drafts_with};
use input_fixtures::{code, key, type_keys};
use mira_assembly::mira_vocabulary::{AccountId, ActionId, Event, EventKind, EventSeq, Timestamp};
use mira_tui::state::{AppState, Popup};
use mira_tui::update::{DraftsIntent, Intent, Msg, update};
use navigation_fixtures::press;

/// Vue ouverte sur les brouillons A, B, C.
fn three_drafts() -> AppState {
    let mut state = state_with_mailboxes();
    open_drafts_with(&mut state, named(&["A", "B", "C"]));
    state
}

/// `D` puis `D` supprime le brouillon sélectionné (lui, par son identifiant) ; le succès relit la vue.
#[test]
fn delete_requires_two_capital_d() {
    let mut state = three_drafts();
    press(&mut state, Intent::MoveDown);
    assert!(type_keys(&mut state, &[key('D')]).is_empty());
    assert!(
        matches!(&state.popup, Popup::ConfirmDeleteDraft { draft, .. } if draft.as_str() == "B")
    );
    assert_eq!(
        type_keys(&mut state, &[key('D')]),
        vec![delete_command("B")]
    );
    assert_eq!(state.popup, Popup::Closed);
    assert_eq!(state.drafts.items().len(), 3);
    let commands = update(&mut state, deleted_reply("B", Ok(())));
    assert_eq!(commands, vec![list_command("a1", 2)]);
    let shown = state.status.message.clone().unwrap();
    assert!(
        shown.contains("Objet B") && shown.contains("supprimé"),
        "{shown}"
    );
}

/// `D` suivi de n'importe quelle autre touche (x, Entrée, Échap, d, Ctrl+D, Alt+D) annule sans rien détruire.
#[test]
fn delete_cancelled_by_any_other_key() {
    let mut state = three_drafts();
    let others = [
        key('x'),
        code(KeyCode::Enter),
        code(KeyCode::Esc),
        key('d'),
        key('y'),
        KeyEvent::new(
            KeyCode::Char('D'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ),
        KeyEvent::new(KeyCode::Char('D'), KeyModifiers::ALT | KeyModifiers::SHIFT),
    ];
    for other in others {
        let commands = type_keys(&mut state, &[key('D'), other]);
        assert!(commands.is_empty(), "{other:?} : {commands:?}");
        assert_eq!(state.popup, Popup::Closed, "{other:?}");
        assert!(state.drafts.is_open(), "{other:?}");
        assert_eq!(state.drafts.items().len(), 3);
    }
}

/// Le moteur refuse la suppression (brouillon planifié) : son erreur est affichée, rien ne quitte la liste.
#[test]
fn delete_refused_by_engine_keeps_list() {
    let mut state = three_drafts();
    assert_eq!(
        type_keys(&mut state, &[key('D'), key('D')]),
        vec![delete_command("A")]
    );
    let refused = not_editable_error();
    let commands = update(
        &mut state,
        compose_fixtures::deleted_reply("A", Err(refused)),
    );
    assert!(commands.is_empty(), "{commands:?}");
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("planifié"), "{shown}");
    assert_eq!(state.drafts.items().len(), 3);
    assert_eq!(state.drafts.selected().unwrap().id.as_str(), "A");
}

/// Le brouillon sélectionné a disparu d'une relecture : ni `D` ni Entrée ne visent un autre brouillon.
#[test]
fn vanished_selection_never_targets_another_draft() {
    let mut state = three_drafts();
    let event = Msg::Event(Event {
        seq: EventSeq(2),
        at: Timestamp::from_millis(0),
        kind: EventKind::ActionCancelled {
            account: AccountId::from_raw("a1"),
            action: ActionId::from_raw("x"),
        },
    });
    update(&mut state, event);
    update(&mut state, drafts_reply(2, named(&["B", "C"])));
    assert!(state.drafts.selected().is_none());
    assert!(press(&mut state, Intent::Drafts(DraftsIntent::AskDelete)).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("Ce brouillon n'existe plus"), "{shown}");
    state.status.message = None;
    assert!(press(&mut state, Intent::Open).is_empty());
    assert!(
        state
            .status
            .message
            .clone()
            .unwrap()
            .contains("n'existe plus")
    );
    press(&mut state, Intent::MoveDown);
    assert!(state.drafts.selected().is_some());
}

/// L'erreur hostile d'un refus de suppression est neutralisée avant d'atteindre la barre d'état.
#[test]
fn refused_delete_error_is_neutralized_on_screen() {
    let mut state = three_drafts();
    type_keys(&mut state, &[key('D'), key('D')]);
    let mut refused = not_editable_error();
    refused.message = "refus\u{1b}[31m\u{202e}moteur".to_string();
    update(&mut state, deleted_reply("A", Err(refused)));
    let text = compose_fixtures::screen(&state, 140, 30).0;
    assert!(
        !text.contains('\u{1b}') && !text.contains('\u{202e}'),
        "{text}"
    );
    assert!(text.contains("refusmoteur"), "{text}");
}
