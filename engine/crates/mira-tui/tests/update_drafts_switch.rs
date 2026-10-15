//! Vue « Brouillons » : un changement de compte ferme la vue et sa confirmation de suppression, sans rien supprimer.
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
use drafts_fixtures::{named, open_drafts_with};
use input_fixtures::{key, type_keys};
use mira_tui::state::Popup;
use mira_tui::update::{DraftsIntent, Intent};
use navigation_fixtures::press;

/// La confirmation de suppression ne survit pas au changement de compte ; un `D` ensuite ne supprime rien.
#[test]
fn account_switch_closes_delete_confirmation() {
    let mut state = state_with_mailboxes();
    open_drafts_with(&mut state, named(&["A"]));
    type_keys(&mut state, &[key('D')]);
    assert!(matches!(state.popup, Popup::ConfirmDeleteDraft { .. }));
    sidebar_fixtures::refresh(&mut state, &["a2"]);
    assert!(!state.drafts.is_open());
    assert_eq!(state.popup, Popup::Closed);
    let commands = press(&mut state, Intent::Drafts(DraftsIntent::ConfirmDelete));
    assert!(commands.is_empty(), "{commands:?}");
}

/// Fermer la vue pendant une réouverture efface « Ouverture du brouillon… ».
#[test]
fn closing_during_reopen_clears_the_notice() {
    let mut state = state_with_mailboxes();
    open_drafts_with(&mut state, named(&["A"]));
    press(&mut state, Intent::Open);
    assert!(state.status.message.clone().unwrap().contains("Ouverture"));
    press(&mut state, Intent::Back);
    assert_eq!(state.status.message, None);
}
