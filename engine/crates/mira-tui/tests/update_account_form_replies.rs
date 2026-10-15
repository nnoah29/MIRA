//! Réponses du moteur à l'ajout : l'échec garde le formulaire ouvert sans secret, le succès recharge, le tardif est ignoré.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod account_form_fixtures;
mod input_fixtures;
mod sidebar_fixtures;

use account_form_fixtures::{
    SECRET, added_reply, hit, known_provider, login_error, prepared_reply, ready_state, token_of,
    type_text,
};
use crossterm::event::KeyCode;
use mira_assembly::api::{AccountRequest, Request};
use mira_tui::commands::{Command, Tag};
use mira_tui::state::account_form::Phase;
use mira_tui::update::update;
use sidebar_fixtures::account;

/// Un refus de connexion laisse le formulaire ouvert, montre le message du moteur et ne garde aucun secret.
#[test]
fn connection_error_keeps_form_open_without_secret() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    let sent = hit(&mut state, KeyCode::Enter);
    let commands = update(&mut state, added_reply(token_of(&sent), Err(login_error())));
    assert!(commands.is_empty());
    let form = state.account_form.form().unwrap();
    assert_eq!(form.phase, Phase::Ready);
    let message = form.error.clone().unwrap();
    assert!(
        message.contains("identifiants refusés par le serveur"),
        "{message}"
    );
    assert_eq!(form.password.char_count(), 0);
    assert!(!format!("{state:?}{commands:?}").contains(SECRET));
}

/// Un succès ferme le formulaire, annonce le compte et relit la liste des comptes.
#[test]
fn successful_add_closes_form_and_reloads_accounts() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    let sent = hit(&mut state, KeyCode::Enter);
    let commands = update(&mut state, added_reply(token_of(&sent), Ok(account("a1"))));
    assert!(!state.account_form.is_open());
    assert_eq!(
        state.status.message.as_deref(),
        Some("Compte ajouté : a1@example.org")
    );
    let reload = Command::Execute {
        tag: Tag::Accounts,
        request: Request::Account(AccountRequest::ListAccounts),
    };
    assert_eq!(commands, vec![reload]);
}

/// Une réponse d'un autre jeton, ou arrivée après la fermeture, ne change rien.
#[test]
fn late_replies_are_ignored() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    let sent = hit(&mut state, KeyCode::Enter);
    let token = token_of(&sent);
    update(&mut state, added_reply(token + 1, Ok(account("a1"))));
    assert!(state.account_form.is_open());
    assert_eq!(
        state.account_form.form().unwrap().phase,
        Phase::Adding(token)
    );
    hit(&mut state, KeyCode::Esc);
    hit(&mut state, KeyCode::Enter);
    assert!(!state.account_form.is_open());
    let commands = update(&mut state, added_reply(token, Ok(account("a1"))));
    assert!(commands.is_empty());
    assert_eq!(
        state
            .status
            .message
            .as_deref()
            .map(|m| m.contains("Compte ajouté")),
        Some(false)
    );
}

/// Un échec de préparation garde le formulaire ouvert sur l'adresse, avec le message du moteur.
#[test]
fn prepare_error_keeps_form_open_on_the_address() {
    let mut state = mira_tui::state::AppState::new(100, 30, 1);
    hit(&mut state, KeyCode::Char('n'));
    type_text(&mut state, "pas-une-adresse");
    let sent = hit(&mut state, KeyCode::Enter);
    let reply = mira_tui::update::Msg::Reply(mira_tui::update::Reply {
        tag: Tag::Prepared(token_of(&sent)),
        result: Err(login_error()),
    });
    update(&mut state, reply);
    let form = state.account_form.form().unwrap();
    assert_eq!(form.phase, Phase::Typing);
    assert!(form.error.is_some());
    let late = prepared_reply(token_of(&sent), known_provider());
    update(&mut state, late);
    assert!(state.account_form.form().unwrap().prepared.is_none());
}
