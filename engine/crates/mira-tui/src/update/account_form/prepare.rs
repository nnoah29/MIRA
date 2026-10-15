//! Préparation de l'adresse : le moteur propose le fournisseur, ses méthodes d'authentification et ses serveurs.

use mira_assembly::api::{AccountRequest, Request, Response};
use mira_assembly::mira_vocabulary::MiraError;

use crate::commands::{Command, Tag};
use crate::state::AppState;
use crate::state::account_form::{AccountForm, Phase, Prepared};
use crate::view::safe_text::display_safe;

/// Demande la préparation de l'adresse saisie ; une adresse vide est refusée sans requête.
pub fn start_prepare(state: &mut AppState) -> Vec<Command> {
    let address = match state.account_form.form() {
        Some(form) => form.address.text().trim().to_string(),
        None => return Vec::new(),
    };
    let token = state.account_form.next_token();
    let Some(form) = state.account_form.form_mut() else {
        return Vec::new();
    };
    if address.is_empty() {
        form.error = Some("Saisissez une adresse e-mail".to_string());
        return Vec::new();
    }
    form.error = None;
    form.phase = Phase::Preparing(token);
    let request = AccountRequest::PrepareAccount {
        address,
        provider: None,
    };
    vec![Command::Execute {
        tag: Tag::Prepared(token),
        request: Request::Account(request),
    }]
}

/// Range la préparation si c'est celle attendue par le formulaire ouvert ; un échec garde le formulaire ouvert.
pub fn on_prepared(
    state: &mut AppState,
    token: u64,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    let Some(form) = state.account_form.form_mut() else {
        return Vec::new();
    };
    if form.phase != Phase::Preparing(token) {
        return Vec::new();
    }
    match result {
        Ok(Response::AccountPreparation(found)) => {
            form.apply_preparation(Prepared::new(found.provider, found.servers));
        }
        Ok(_) => fail(form, "réponse inattendue du moteur".to_string()),
        Err(error) => fail(form, display_safe(&error.to_string())),
    }
    Vec::new()
}

/// Revient à la saisie de l'adresse avec ce message.
fn fail(form: &mut AccountForm, message: String) {
    form.phase = Phase::Typing;
    form.error = Some(message);
}
