//! Actions sur le message sélectionné : envoi, confirmation de la suppression, mise à jour optimiste.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::{action_command, state_with_mailboxes};
use mira_assembly::mira_vocabulary::ActionKind;
use mira_tui::state::Popup;
use mira_tui::update::Intent;
use navigation_fixtures::press;

/// Archiver envoie `Archive` pour le message sélectionné et le retire de la liste.
#[test]
fn archive_sends_apply_action_archive_for_selected() {
    let mut state = state_with_mailboxes();
    let commands = press(&mut state, Intent::Archive);
    assert_eq!(
        commands,
        vec![action_command(ActionKind::Archive, &["m1"], "tui-1-act-1")]
    );
    assert_eq!(state.messages.len(), 1);
    assert_eq!(state.messages.items()[0].id.to_string(), "m2");
}

/// La suppression définitive n'envoie rien tant qu'elle n'est pas confirmée.
#[test]
fn permanent_delete_requires_confirmation() {
    let mut state = state_with_mailboxes();
    let commands = press(&mut state, Intent::DeletePermanently);
    assert!(commands.is_empty());
    assert!(matches!(state.popup, Popup::ConfirmDelete { .. }));
    assert_eq!(state.messages.len(), 2);
}

/// Confirmer envoie l'action ; annuler ferme la fenêtre sans rien envoyer.
#[test]
fn confirm_then_sends_action_cancel_sends_nothing() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::DeletePermanently);
    assert!(press(&mut state, Intent::Cancel).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    assert_eq!(state.messages.len(), 2);
    press(&mut state, Intent::DeletePermanently);
    let commands = press(&mut state, Intent::Confirm);
    let wanted = action_command(ActionKind::DeletePermanently, &["m1"], "tui-1-act-1");
    assert_eq!(commands, vec![wanted]);
    assert_eq!(state.popup, Popup::Closed);
    assert_eq!(state.messages.len(), 1);
}

/// Chaque action porte une clé d'idempotence neuve.
#[test]
fn each_action_uses_a_fresh_idempotency_key() {
    let mut state = state_with_mailboxes();
    let first = press(&mut state, Intent::ToggleFlag);
    let second = press(&mut state, Intent::ToggleFlag);
    assert_eq!(
        first,
        vec![action_command(
            ActionKind::MarkFlagged { flagged: true },
            &["m1"],
            "tui-1-act-1"
        )]
    );
    assert_eq!(
        second,
        vec![action_command(
            ActionKind::MarkFlagged { flagged: false },
            &["m1"],
            "tui-1-act-2"
        )]
    );
}

/// Basculer « lu » inverse l'état et envoie l'action correspondante sans confirmation.
#[test]
fn toggle_read_flips_and_sends() {
    let mut state = state_with_mailboxes();
    let commands = press(&mut state, Intent::ToggleRead);
    let wanted = action_command(ActionKind::MarkRead { read: true }, &["m1"], "tui-1-act-1");
    assert_eq!(commands, vec![wanted]);
    assert!(state.messages.items()[0].flags.seen);
}

/// Sans message sélectionné, aucune action n'est envoyée.
#[test]
fn actions_without_selection_do_nothing() {
    let mut state = mira_tui::state::AppState::new(100, 30, 1);
    assert!(press(&mut state, Intent::Archive).is_empty());
    assert!(press(&mut state, Intent::Trash).is_empty());
    assert!(press(&mut state, Intent::DeletePermanently).is_empty());
    assert_eq!(state.popup, Popup::Closed);
}
