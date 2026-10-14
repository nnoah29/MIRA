//! Rendu de la barre latérale, de l'accueil sans compte et de la barre d'état.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod sidebar_fixtures;

use mira_assembly::mira_vocabulary::MailboxRole;
use mira_tui::state::AppState;
use sidebar_fixtures::{account, mailbox, rendered_text};

/// Sans compte, l'écran d'accueil invite à ajouter un compte avec `mira account add` (la touche n viendra à l'étape 3).
#[test]
fn no_accounts_shows_welcome_with_add_hint() {
    let text = rendered_text(&AppState::new(100, 30, 1));
    assert!(text.contains("ajouter un compte"));
    assert!(text.contains("« mira account add »"), "{text}");
    assert!(text.contains("Aucun compte"));
}

/// Les boîtes sont listées avec leur nombre de non lus.
#[test]
fn sidebar_lists_mailboxes_with_unread_counts() {
    let mut state = AppState::new(100, 30, 1);
    state.accounts.replace_items(vec![account("a1")]);
    state.mailboxes.replace_items(vec![
        mailbox("a1", "INBOX", MailboxRole::Inbox, 12),
        mailbox("a1", "Sent", MailboxRole::Sent, 0),
    ]);
    let text = rendered_text(&state);
    assert!(text.contains("Réception 12"));
    assert!(text.contains("Envoyés"));
    assert!(text.contains("a1@example.org"));
}

/// Un compte sans boîte s'affiche sans panique.
#[test]
fn account_without_mailbox_renders() {
    let mut state = AppState::new(100, 30, 1);
    state.accounts.replace_items(vec![account("a1")]);
    let text = rendered_text(&state);
    assert!(text.contains("a1@example.org"));
}

/// Le message de la barre d'état est affiché en bas de l'écran.
#[test]
fn status_message_is_shown_on_last_line() {
    let mut state = AppState::new(100, 30, 1);
    state.accounts.replace_items(vec![account("a1")]);
    state.status.message = Some("serveur injoignable".to_string());
    let text = rendered_text(&state);
    let last = text.lines().last().unwrap();
    assert!(last.contains("serveur injoignable"));
}

/// Sans compte, une erreur de chargement s'affiche sous l'accueil.
#[test]
fn welcome_keeps_status_line_for_errors() {
    let mut state = AppState::new(100, 30, 1);
    state.status.message = Some("serveur injoignable".to_string());
    let text = rendered_text(&state);
    assert!(text.contains("ajouter un compte"));
    assert!(text.lines().last().unwrap().contains("serveur injoignable"));
}
