//! Réponse du moteur à l'ajout : succès ferme le formulaire et recharge les comptes, échec le laisse ouvert.

use mira_assembly::api::{AccountRequest, Request, Response};
use mira_assembly::mira_vocabulary::MiraError;

use crate::commands::{Command, Tag};
use crate::state::AppState;
use crate::state::account_form::Phase;
use crate::view::safe_text::display_safe;

/// Traite la réponse de l'ajout attendue par le formulaire ouvert ; toute autre est ignorée.
pub fn on_account_added(
    state: &mut AppState,
    token: u64,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    let waiting = state
        .account_form
        .form()
        .is_some_and(|form| form.phase == Phase::Adding(token));
    if !waiting {
        return Vec::new();
    }
    match result {
        Ok(Response::Account(account)) => {
            state.account_form.close();
            let address = display_safe(account.address.as_str());
            state.status.message = Some(format!("Compte ajouté : {address}"));
            reload_accounts(state)
        }
        Ok(_) => keep_open(state, "réponse inattendue du moteur".to_string()),
        Err(error) => keep_open(state, display_safe(&error.to_string())),
    }
}

/// Garde le formulaire ouvert avec l'erreur ; le mot de passe est parti, il faut le saisir de nouveau.
fn keep_open(state: &mut AppState, message: String) -> Vec<Command> {
    if let Some(form) = state.account_form.form_mut() {
        form.phase = Phase::Ready;
        form.error = Some(format!("{message} — saisissez de nouveau le mot de passe"));
    }
    Vec::new()
}

/// Relit les comptes pour que le nouveau apparaisse dans la barre latérale, sauf si une relecture est déjà en cours.
fn reload_accounts(state: &mut AppState) -> Vec<Command> {
    if state.refresh.accounts {
        return Vec::new();
    }
    state.refresh.accounts = true;
    vec![Command::Execute {
        tag: Tag::Accounts,
        request: Request::Account(AccountRequest::ListAccounts),
    }]
}
