//! Formulaire d'ajout de compte : `n` l'ouvre, l'adresse déclenche la préparation, les serveurs ne sont demandés qu'au repli.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod account_form_fixtures;
mod input_fixtures;

use account_form_fixtures::{
    fallback_provider, hit, known_provider, prepared_reply, ready_state, type_text,
};
use crossterm::event::KeyCode;
use input_fixtures::{key, type_keys};
use mira_assembly::api::{AccountRequest, Request};
use mira_tui::commands::{Command, Tag};
use mira_tui::state::AppState;
use mira_tui::state::account_form::{FormField, Phase};
use mira_tui::update::update;

/// `n` ouvre le formulaire, l'adresse tapée puis Entrée envoie `PrepareAccount` suivi par un jeton.
#[test]
fn address_triggers_prepare_account() {
    let mut state = AppState::new(100, 30, 1);
    assert!(type_keys(&mut state, &[key('n')]).is_empty());
    assert!(state.account_form.is_open());
    assert!(type_text(&mut state, "alice@acme.test").is_empty());
    let commands = hit(&mut state, KeyCode::Enter);
    let request = Request::Account(AccountRequest::PrepareAccount {
        address: "alice@acme.test".to_string(),
        provider: None,
    });
    let expected = Command::Execute {
        tag: Tag::Prepared(1),
        request,
    };
    assert_eq!(commands, vec![expected]);
    assert_eq!(
        state.account_form.form().unwrap().phase,
        Phase::Preparing(1)
    );
}

/// Les champs de serveurs n'existent que pour le fournisseur de repli (sans domaine propre).
#[test]
fn manual_servers_asked_only_for_generic_provider() {
    let fallback = ready_state(fallback_provider());
    let fields = fallback.account_form.form().unwrap().visible_fields();
    for wanted in [FormField::ImapHost, FormField::ImapPort, FormField::SmtpHost, FormField::SmtpPort] {
        assert!(fields.contains(&wanted), "{wanted:?} absent : {fields:?}");
    }
    assert_eq!(fallback.account_form.form().unwrap().focus, FormField::ImapHost);
    let known = ready_state(known_provider());
    let fields = known.account_form.form().unwrap().visible_fields();
    assert!(!fields.contains(&FormField::ImapHost), "{fields:?}");
    assert_eq!(known.account_form.form().unwrap().focus, FormField::Password);
}

/// Une adresse vide n'envoie rien et le dit dans le formulaire.
#[test]
fn empty_address_sends_nothing() {
    let mut state = AppState::new(100, 30, 1);
    type_keys(&mut state, &[key('n')]);
    assert!(hit(&mut state, KeyCode::Enter).is_empty());
    let form = state.account_form.form().unwrap();
    assert_eq!(form.phase, Phase::Typing);
    assert!(form.error.is_some());
}

/// Modifier l'adresse déjà préparée annule la préparation et efface le mot de passe saisi.
#[test]
fn editing_a_prepared_address_resets_the_preparation() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, "mot-de-passe");
    hit(&mut state, KeyCode::BackTab);
    hit(&mut state, KeyCode::BackTab);
    hit(&mut state, KeyCode::BackTab);
    assert_eq!(state.account_form.form().unwrap().focus, FormField::Address);
    type_text(&mut state, "x");
    let form = state.account_form.form().unwrap();
    assert!(form.prepared.is_none());
    assert_eq!(form.phase, Phase::Typing);
    assert_eq!(form.password.char_count(), 0);
}

/// Pendant la préparation la frappe est ignorée et la réponse d'un autre jeton aussi.
#[test]
fn typing_waits_for_the_prepared_reply() {
    let mut state = AppState::new(100, 30, 1);
    type_keys(&mut state, &[key('n')]);
    type_text(&mut state, "a@acme.test");
    hit(&mut state, KeyCode::Enter);
    type_text(&mut state, "zzz");
    assert_eq!(state.account_form.form().unwrap().address.text(), "a@acme.test");
    update(&mut state, prepared_reply(99, known_provider()));
    assert!(state.account_form.form().unwrap().prepared.is_none());
    update(&mut state, prepared_reply(1, known_provider()));
    assert!(state.account_form.form().unwrap().prepared.is_some());
}
