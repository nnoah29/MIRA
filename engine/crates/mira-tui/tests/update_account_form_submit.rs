//! Envoi de l'ajout de compte : requête exacte, mot de passe vidé de l'état, clé stable, pas de double envoi.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod account_form_fixtures;
mod input_fixtures;

use account_form_fixtures::{
    SECRET, added_command, added_reply, hit, known_provider, login_error, oauth_only_provider,
    ready_state, token_of, type_text,
};
use crossterm::event::KeyCode;
use mira_assembly::api::{AccountRequest, CredentialInput, SecretInput};
use mira_assembly::mira_vocabulary::{AuthMethodKind, IdempotencyKey, ProviderId};
use mira_tui::state::account_form::Phase;
use mira_tui::update::update;

/// Le mot de passe part dans la requête et n'est plus dans l'état : ni champ, ni `Debug`.
#[test]
fn secret_cleared_from_state_after_submit() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    let commands = hit(&mut state, KeyCode::Enter);
    let AccountRequest::AddAccount { credential, .. } = added_command(&commands) else {
        panic!("AddAccount attendu");
    };
    assert_eq!(credential, CredentialInput::Password(SecretInput::new(SECRET)));
    let form = state.account_form.form().unwrap();
    assert_eq!(form.password.char_count(), 0);
    assert_eq!(form.phase, Phase::Adding(token_of(&commands)));
    assert!(!format!("{state:?}").contains(SECRET));
}

/// Le brouillon porte l'adresse, le nom par défaut, le fournisseur vu par l'utilisateur et une clé de session.
#[test]
fn submit_sends_add_account_with_session_key() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    let commands = hit(&mut state, KeyCode::Enter);
    let AccountRequest::AddAccount { draft, idempotency_key, .. } = added_command(&commands) else {
        panic!("AddAccount attendu");
    };
    assert_eq!(draft.address, "alice@acme.test");
    assert_eq!(draft.display_name, "alice@acme.test");
    assert_eq!(draft.provider, Some(ProviderId::new("acme")));
    assert_eq!(draft.servers, None);
    assert_eq!(draft.auth, AuthMethodKind::Password);
    assert_eq!(idempotency_key, IdempotencyKey("tui-7-add-account-1".to_string()));
}

/// Après un refus, le renvoi de la même adresse réutilise la même clé d'idempotence.
#[test]
fn retry_after_error_reuses_the_same_key() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    let first = hit(&mut state, KeyCode::Enter);
    update(&mut state, added_reply(token_of(&first), Err(login_error())));
    type_text(&mut state, SECRET);
    let second = hit(&mut state, KeyCode::Enter);
    let key_of = |commands: &[_]| match added_command(commands) {
        AccountRequest::AddAccount { idempotency_key, .. } => idempotency_key,
        other => panic!("{other:?}"),
    };
    assert_eq!(key_of(&first), key_of(&second));
    assert_ne!(token_of(&first), token_of(&second));
}

/// Sans mot de passe rien n'est envoyé et le formulaire le dit.
#[test]
fn missing_password_sends_nothing() {
    let mut state = ready_state(known_provider());
    assert!(hit(&mut state, KeyCode::Enter).is_empty());
    let form = state.account_form.form().unwrap();
    assert_eq!(form.error.as_deref(), Some("Saisissez le mot de passe"));
    assert_eq!(form.phase, Phase::Ready);
}

/// Un fournisseur qui ne propose qu'OAuth ne peut pas être envoyé dans cette version, et le formulaire le dit.
#[test]
fn oauth_only_provider_cannot_submit() {
    let mut state = ready_state(oauth_only_provider());
    assert!(hit(&mut state, KeyCode::Enter).is_empty());
    let form = state.account_form.form().unwrap();
    assert_eq!(
        form.error.as_deref(),
        Some("OAuth : non disponible dans cette version")
    );
    assert_eq!(form.visible_fields().len(), 1);
    assert_eq!(form.phase, Phase::Ready);
}

/// Une seconde touche Entrée pendant l'ajout n'envoie rien : un seul `AddAccount` part.
#[test]
fn second_enter_while_adding_sends_nothing() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    assert_eq!(hit(&mut state, KeyCode::Enter).len(), 1);
    type_text(&mut state, "autre");
    assert!(hit(&mut state, KeyCode::Enter).is_empty());
    assert_eq!(state.account_form.form().unwrap().password.char_count(), 0);
}
