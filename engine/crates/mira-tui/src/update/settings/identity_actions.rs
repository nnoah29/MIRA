//! Page Identités : lecture, ouverture des formulaires d'ajout et de modification, identité par défaut.

use mira_assembly::api::{IdentityRequest, Request, Response};
use mira_assembly::mira_vocabulary::MiraError;

use super::account_nav::current_account;
use super::intent::IdentityOp;
use super::operation::{say, start_operation};
use crate::commands::{Command, SettingsTag, Tag};
use crate::state::AppState;
use crate::state::settings::{Dialog, Field, Form, FormKind, Operation, SettingsTab};
use crate::update::accounts_reply::{report, unexpected};

/// Applique une opération de la page Identités ; sur une autre page elle est ignorée.
pub fn on_identity_op(state: &mut AppState, op: IdentityOp) -> Vec<Command> {
    if state.settings.tab != SettingsTab::Identities || state.settings.dialog != Dialog::None {
        return Vec::new();
    }
    match op {
        IdentityOp::Add => ask_add(state),
        IdentityOp::Edit => ask_edit(state),
        IdentityOp::SetDefault => set_default(state),
    }
}

/// Lit les identités du compte visé ; la liste d'un autre compte est d'abord vidée.
pub fn reload_identities(state: &mut AppState) -> Vec<Command> {
    let Some(account) = current_account(state).map(|account| account.id.clone()) else {
        return Vec::new();
    };
    state.settings.identities.prepare_for(&account);
    let token = state.settings.begin_identities();
    vec![Command::Execute {
        tag: Tag::Settings(SettingsTag::Identities(token)),
        request: Request::Identity(IdentityRequest::ListIdentities { account }),
    }]
}

/// Réponse à la lecture des identités : prise seulement si c'est la lecture attendue de la vue ouverte.
pub fn on_identities(
    state: &mut AppState,
    token: u64,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if !state.settings.take_identities(token) {
        return Vec::new();
    }
    match result {
        Ok(Response::Identities(items)) => state.settings.identities.replace(items),
        Ok(_) => return unexpected(state),
        Err(error) => return report(state, error),
    }
    Vec::new()
}

/// Ouvre le formulaire d'ajout d'une identité au compte visé.
fn ask_add(state: &mut AppState) -> Vec<Command> {
    let Some(account) = current_account(state).map(|account| account.id.clone()) else {
        return say(state, "Aucun compte");
    };
    let fields = vec![
        Field::new("Adresse", ""),
        Field::new("Nom affiché", ""),
        Field::new("Signature", ""),
    ];
    state.settings.dialog = Dialog::Form(Form::new(FormKind::AddIdentity(account), fields));
    Vec::new()
}

/// Ouvre le formulaire de modification de l'identité sélectionnée.
fn ask_edit(state: &mut AppState) -> Vec<Command> {
    let Some(identity) = state.settings.identities.selected().cloned() else {
        return say(state, "Aucune identité sélectionnée");
    };
    let signature = identity.signature.clone().unwrap_or_default();
    let fields = vec![
        Field::new("Nom affiché", &identity.display_name),
        Field::new("Signature", &signature),
    ];
    state.settings.dialog = Dialog::Form(Form::new(FormKind::EditIdentity(identity.id), fields));
    Vec::new()
}

/// Demande que l'identité sélectionnée devienne celle par défaut du compte.
fn set_default(state: &mut AppState) -> Vec<Command> {
    let Some(identity) = state.settings.identities.selected().cloned() else {
        return say(state, "Aucune identité sélectionnée");
    };
    if identity.is_default {
        return say(state, "Cette identité est déjà celle par défaut");
    }
    let request = IdentityRequest::SetDefaultIdentity {
        identity: identity.id,
    };
    start_operation(
        state,
        Operation::SetDefaultIdentity,
        Request::Identity(request),
    )
}
