//! Le texte venant du serveur est neutralisé aussi dans la barre latérale et la barre d'état.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod sidebar_fixtures;

use mira_assembly::mira_vocabulary::{
    AccountId, EmailAddress, ErrorCategory, MailboxRole, MiraError,
};
use mira_tui::commands::Tag;
use mira_tui::state::AppState;
use mira_tui::update::{Msg, Reply, update};
use sidebar_fixtures::{account, mailbox, rendered_text};

/// Erreur du moteur dont le texte contient une séquence d'effacement d'écran.
fn hostile_error() -> MiraError {
    MiraError::new(
        ErrorCategory::Network,
        mira_assembly::mira_vocabulary::error::codes::network::UNREACHABLE,
        "refus\u{1b}[2J du serveur\u{7}",
    )
}

/// Le nom d'une boîte de rôle « Autre » est neutralisé.
#[test]
fn other_mailbox_label_is_neutralized() {
    let mut state = AppState::new(100, 30, 1);
    state.accounts.replace_items(vec![account("a1")]);
    let hostile = mailbox("a1", "Pro\u{1b}[2J\u{7}jets", MailboxRole::Other, 0);
    state.mailboxes.replace_items(vec![hostile]);
    let text = rendered_text(&state);
    assert!(text.contains("Projets"));
    assert!(!text.contains("[2J"));
}

/// L'adresse d'un compte passe par le texte neutralisé quand elle est affichée.
#[test]
fn account_address_goes_through_safe_text() {
    let mut state = AppState::new(100, 30, 1);
    let mut hostile = account("a1");
    hostile.address = EmailAddress::parse("ann\u{1b}[2J@example.org").unwrap();
    state.accounts.replace_items(vec![hostile]);
    let text = rendered_text(&state);
    assert!(!text.contains('\u{1b}'));
    assert!(!text.contains("[2J"));
}

/// Une erreur de la liste des comptes est neutralisée dans la barre d'état.
#[test]
fn accounts_error_status_is_neutralized() {
    let mut state = AppState::new(100, 30, 1);
    let reply = Reply {
        tag: Tag::Accounts,
        result: Err(hostile_error()),
    };
    update(&mut state, Msg::Reply(reply));
    let message = state.status.message.clone().unwrap();
    assert!(!message.contains('\u{1b}') && !message.contains('\u{7}'));
    assert!(rendered_text(&state).contains("refus du serveur"));
}

/// Une erreur de la liste des boîtes est neutralisée dans la barre d'état.
#[test]
fn mailboxes_error_status_is_neutralized() {
    let mut state = AppState::new(100, 30, 1);
    let reply = Reply {
        tag: Tag::Mailboxes(AccountId::from_raw("a1")),
        result: Err(hostile_error()),
    };
    update(&mut state, Msg::Reply(reply));
    let message = state.status.message.clone().unwrap();
    assert!(!message.contains('\u{1b}') && !message.contains('\u{7}'));
    assert!(rendered_text(&state).contains("refus du serveur"));
}
