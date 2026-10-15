//! Envoi de l'ajout de compte : le moteur teste la connexion puis enregistre ; le mot de passe quitte l'état avec la requête.

use mira_assembly::api::{AccountRequest, Request};

use super::draft::{build_draft, take_credential};
use crate::commands::{Command, Tag};
use crate::state::AppState;
use crate::state::account_form::Phase;

/// Valide la saisie, tire la clé d'idempotence du formulaire puis envoie `AddAccount` en vidant le mot de passe.
pub fn submit(state: &mut AppState) -> Vec<Command> {
    let drafted = match state.account_form.form().map(build_draft) {
        Some(Ok(draft)) => draft,
        Some(Err(message)) => return refuse(state, message),
        None => return Vec::new(),
    };
    let Some(key) = form_key(state) else {
        return Vec::new();
    };
    let token = state.account_form.next_token();
    let Some(form) = state.account_form.form_mut() else {
        return Vec::new();
    };
    let Some(credential) = take_credential(form, drafted.auth) else {
        return Vec::new();
    };
    form.phase = Phase::Adding(token);
    form.error = None;
    let request = AccountRequest::AddAccount {
        draft: drafted,
        credential,
        idempotency_key: key,
    };
    vec![Command::Execute {
        tag: Tag::AccountAdded(token),
        request: Request::Account(request),
    }]
}

/// Montre pourquoi rien n'est envoyé, dans le formulaire.
fn refuse(state: &mut AppState, message: String) -> Vec<Command> {
    if let Some(form) = state.account_form.form_mut() {
        form.error = Some(message);
    }
    Vec::new()
}

/// Clé d'idempotence de ce formulaire : tirée au premier envoi, réutilisée aux renvois de la même adresse.
fn form_key(state: &mut AppState) -> Option<mira_assembly::mira_vocabulary::IdempotencyKey> {
    let existing = state.account_form.form()?.key.clone();
    let key = match existing {
        Some(key) => key,
        None => state.fresh_key("add-account"),
    };
    state.account_form.form_mut()?.key = Some(key.clone());
    Some(key)
}
