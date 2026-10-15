//! Serveurs saisis à la main pour le fournisseur de repli : valeurs proposées, saisie, validation, chiffrement.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod account_form_fixtures;
mod input_fixtures;

use account_form_fixtures::{
    SECRET, added_command, fallback_provider, hit, known_provider, ready_state, type_text,
};
use crossterm::event::KeyCode;
use input_fixtures::{ctrl, type_keys};
use mira_assembly::api::AccountRequest;
use mira_assembly::mira_vocabulary::Security;
use mira_tui::state::AppState;

/// Place le focus sur le mot de passe du formulaire de repli et le saisit.
fn fill_password(state: &mut AppState) {
    for _ in 0..4 {
        hit(state, KeyCode::Tab);
    }
    type_text(state, SECRET);
}

/// Les serveurs proposés par le moteur partent tels quels si l'utilisateur ne change rien.
#[test]
fn proposed_servers_are_sent_unchanged() {
    let mut state = ready_state(fallback_provider());
    fill_password(&mut state);
    let commands = hit(&mut state, KeyCode::Enter);
    let AccountRequest::AddAccount { draft, .. } = added_command(&commands) else {
        panic!("AddAccount attendu");
    };
    let servers = draft.servers.unwrap();
    assert_eq!(servers, account_form_fixtures::proposed_servers());
}

/// Les valeurs tapées remplacent celles du champ, et Ctrl+t change le chiffrement du serveur du champ.
#[test]
fn typed_servers_and_security_are_sent() {
    let mut state = ready_state(fallback_provider());
    for _ in 0..16 {
        hit(&mut state, KeyCode::Backspace);
    }
    type_text(&mut state, "mail.local");
    type_keys(&mut state, &[ctrl('t')]);
    hit(&mut state, KeyCode::Tab);
    for _ in 0..3 {
        hit(&mut state, KeyCode::Backspace);
    }
    type_text(&mut state, "143");
    hit(&mut state, KeyCode::Tab);
    hit(&mut state, KeyCode::Tab);
    hit(&mut state, KeyCode::Tab);
    type_text(&mut state, SECRET);
    let commands = hit(&mut state, KeyCode::Enter);
    let AccountRequest::AddAccount { draft, .. } = added_command(&commands) else {
        panic!("AddAccount attendu");
    };
    let servers = draft.servers.unwrap();
    assert_eq!(servers.imap.host, "mail.local");
    assert_eq!(servers.imap.port, 143);
    assert_eq!(servers.imap.security, Security::StartTls);
    assert_eq!(servers.smtp.security, Security::Tls);
}

/// Un port invalide ou un hôte vide n'envoie rien et nomme le serveur fautif.
#[test]
fn invalid_server_fields_send_nothing() {
    let mut state = ready_state(fallback_provider());
    hit(&mut state, KeyCode::Tab);
    type_text(&mut state, "abc");
    fill_password_from_port(&mut state);
    assert!(hit(&mut state, KeyCode::Enter).is_empty());
    let message = state.account_form.form().unwrap().error.clone().unwrap();
    assert!(
        message.contains("IMAP") && message.contains("port"),
        "{message}"
    );
}

/// Depuis le port IMAP, atteint le mot de passe et le saisit.
fn fill_password_from_port(state: &mut AppState) {
    for _ in 0..3 {
        hit(state, KeyCode::Tab);
    }
    type_text(state, SECRET);
}

/// Un fournisseur qui a ses serveurs n'en envoie aucun : le moteur utilise les siens.
#[test]
fn known_provider_sends_no_servers() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    let commands = hit(&mut state, KeyCode::Enter);
    let AccountRequest::AddAccount { draft, .. } = added_command(&commands) else {
        panic!("AddAccount attendu");
    };
    assert!(draft.servers.is_none());
}
