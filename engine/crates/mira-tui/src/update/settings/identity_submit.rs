//! Validation des formulaires d'identité : adresse analysée, seuls les champs changés partent.

use mira_assembly::api::{IdentityRequest, NewIdentity, Request};
use mira_assembly::mira_vocabulary::{AccountId, EmailAddress, IdentityId};

use super::operation::{say, submit_operation};
use crate::commands::Command;
use crate::state::AppState;
use crate::state::settings::{Dialog, Form, Operation};

/// Valide le formulaire d'ajout : une adresse invalide laisse le formulaire ouvert et ne part pas.
pub fn submit_add_identity(state: &mut AppState, form: &Form, account: AccountId) -> Vec<Command> {
    let Ok(address) = EmailAddress::parse(form.value(0)) else {
        return say(state, "Adresse invalide");
    };
    let signature = form.value(2).trim();
    let new = NewIdentity {
        account,
        address: address.as_str().to_string(),
        display_name: form.value(1).trim().to_string(),
        signature: (!signature.is_empty()).then(|| signature.to_string()),
        is_default: false,
    };
    let request = Request::Identity(IdentityRequest::AddIdentity(new));
    submit_operation(state, Operation::AddIdentity, request)
}

/// Valide le formulaire de modification : seuls le nom et la signature changés partent.
pub fn submit_edit_identity(
    state: &mut AppState,
    form: &Form,
    identity: IdentityId,
) -> Vec<Command> {
    if !form.changed(0) && !form.changed(1) {
        state.settings.dialog = Dialog::None;
        return say(state, "Aucune modification");
    }
    let request = IdentityRequest::UpdateIdentity {
        identity,
        display_name: form.changed(0).then(|| form.value(0).trim().to_string()),
        signature: form.changed(1).then(|| form.value(1).trim().to_string()),
    };
    let request = Request::Identity(request);
    submit_operation(state, Operation::UpdateIdentity, request)
}
