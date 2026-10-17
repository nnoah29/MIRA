//! Arrêt sans `q` (signal, clavier fermé) : les modifications du brouillon ouvert sont enregistrées, jamais supprimées.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{open_modified, open_reply, subject_only, update_command};
use mira_tui::update::shutdown::shutdown_commands;

/// Brouillon modifié : l'arrêt enregistre les champs modifiés, sans supprimer le brouillon.
#[test]
fn shutdown_saves_a_modified_draft() {
    let mut state = open_modified();
    let commands = shutdown_commands(&mut state);
    assert_eq!(
        commands,
        vec![update_command("d1", subject_only("Re: Bonjour!"))]
    );
}

/// Brouillon intact ou aucun brouillon : l'arrêt n'émet rien (en particulier aucune suppression).
#[test]
fn shutdown_without_changes_emits_nothing() {
    let mut state = open_reply();
    assert!(shutdown_commands(&mut state).is_empty());
    let mut idle = action_fixtures::state_with_mailboxes();
    assert!(shutdown_commands(&mut idle).is_empty());
}
